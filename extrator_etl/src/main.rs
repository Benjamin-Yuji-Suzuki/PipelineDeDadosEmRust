use reqwest::Client;
use serde_json::Value;
use polars::prelude::*;
use rusqlite::{params, Connection};
use std::fs::File;
use std::io::Write;
use chrono::Local;

#[tokio::main]
async fn main() {
    println!("🚀 Iniciando ETL: E-commerce Fake Store...\n");

    // ==========================================
    // ETAPA 1: EXTRACT (Extração - Camada Bronze)
    // ==========================================
    println!("📥 Extraindo dados brutos da API...");
    let client = Client::new();
    let url = "https://fakestoreapi.com/products";
    let resposta = client.get(url).send().await.expect("❌ Erro na API").json::<Value>().await.expect("❌ Erro no JSON");

    let nome_csv = "bronze_produtos.csv";
    let mut arquivo_csv = File::create(nome_csv).expect("❌ Erro ao criar CSV");
    
    writeln!(arquivo_csv, "id,title,category,price,rate").unwrap();

    if let Some(produtos) = resposta.as_array() {
        for p in produtos {
            let id = p["id"].as_i64().unwrap_or(0);
            let title = p["title"].as_str().unwrap_or("").replace(",", "").replace("\n", "");
            let category = p["category"].as_str().unwrap_or("");
            let price = p["price"].as_f64().unwrap_or(0.0);
            let rate = p.get("rating").and_then(|r| r.get("rate")).and_then(|r| r.as_f64()).unwrap_or(0.0);

            writeln!(arquivo_csv, "{},{},{},{},{}", id, title, category, price, rate).unwrap();
        }
    }
    println!("✅ Dados extraídos e salvos em '{}'", nome_csv);

    // ==========================================
    // ISOLAMENTO DE THREAD (A mágica que evita o Panic)
    // Tudo que é "pesado" e síncrono (Polars e SQLite) vai aqui dentro
    // ==========================================
    tokio::task::spawn_blocking(move || {
        
        // ==========================================
        // ETAPA 2: TRANSFORM (Transformação - Camada Silver)
        // ==========================================
        println!("\n⚙️ Iniciando Transformação (Limpeza e Padronização) com Polars...");
        
        let mut df_silver = LazyCsvReader::new(nome_csv.into())
            .with_has_header(true)
            .finish()
            .expect("❌ Erro ao ler o CSV da camada Bronze")
            .with_column(col("id").cast(DataType::Int64))
            .with_column(col("price").cast(DataType::Float64))
            .with_column(col("rate").cast(DataType::Float64))
            .with_column((col("price") * lit(5.0)).alias("preco_brl"))
            .with_column(col("category").str().to_uppercase().alias("categoria_padrao"))
            .filter(col("rate").gt(lit(3.0)))
            .collect()
            .expect("❌ Erro ao processar as transformações no DataFrame");

        println!("✅ Transformação concluída! Sobraram {} produtos com boa avaliação.", df_silver.height());

        // ==========================================
        // ETAPA 3: LOAD (Carregamento - Camada Gold)
        // ==========================================
        println!("\n💾 Iniciando a Carga (Load) na Camada Gold...");

        // 3.1: Salvando no formato Parquet (Data Lake)
        let nome_parquet = "gold_produtos.parquet";
        let mut arquivo_parquet = File::create(nome_parquet).expect("❌ Erro ao criar o arquivo Parquet");
        ParquetWriter::new(&mut arquivo_parquet)
            .finish(&mut df_silver)
            .expect("❌ Erro ao escrever os dados no Parquet");
        println!("✅ Arquivo analítico '{}' gerado com sucesso.", nome_parquet);

        // 3.2: Salvando no Banco de Dados Relacional SQLite
        let banco_nome = "gold_ecommerce.db";
        let conn = Connection::open(banco_nome).expect("❌ Erro ao abrir banco de dados");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS produtos_gold (
                id INTEGER PRIMARY KEY,
                titulo TEXT NOT NULL,
                categoria_padrao TEXT NOT NULL,
                preco_brl REAL NOT NULL,
                avaliacao REAL
            )",
            [],
        ).expect("❌ Erro ao criar a tabela");

        conn.execute("DELETE FROM produtos_gold", []).unwrap();

        let ids = df_silver.column("id").unwrap().i64().unwrap();
        let titulos = df_silver.column("title").unwrap().str().unwrap();
        let categorias = df_silver.column("categoria_padrao").unwrap().str().unwrap();
        let precos = df_silver.column("preco_brl").unwrap().f64().unwrap();
        let notas = df_silver.column("rate").unwrap().f64().unwrap();

        for i in 0..df_silver.height() {
            let id = ids.get(i).unwrap();
            let titulo = titulos.get(i).unwrap();
            let categoria = categorias.get(i).unwrap();
            let preco = precos.get(i).unwrap();
            let nota = notas.get(i).unwrap();

            conn.execute(
                "INSERT INTO produtos_gold (id, titulo, categoria_padrao, preco_brl, avaliacao) 
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, titulo, categoria, preco, nota],
            ).expect("❌ Erro ao inserir dado no banco");
        }
        
        println!("✅ Banco de dados relacional '{}' alimentado com sucesso.", banco_nome);

        let agora = Local::now().format("%d/%m/%Y %H:%M:%S").to_string();
        println!("\n🎉 SUCESSO ABSOLUTO! Pipeline finalizada às {}. Todos os entregáveis estão prontos.", agora);

    }).await.unwrap(); // Espera a thread pesada terminar com segurança
}