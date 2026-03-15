use reqwest::Client;
use serde_json::Value;
use polars::prelude::*;
use std::fs::File;
use std::io::Write;
use chrono::Local;

#[tokio::main]
async fn main() {
    println!("🚀 Iniciando a Pipeline de Extração ETL...\n");

    // ==========================================
    // ETAPA 1: EXTRAÇÃO ASSÍNCRONA DA WEB
    // ==========================================
    let client = Client::new();
    let moedas = vec!["USD-BRL", "EUR-BRL", "BTC-BRL", "GBP-BRL"];
    let mut tarefas = vec![];

    for moeda in moedas {
        let client_clone = client.clone();
        let moeda_string = moeda.to_string();
        
        let tarefa = tokio::spawn(async move {
            let url = format!("https://economia.awesomeapi.com.br/last/{}", moeda_string);
            
            match client_clone.get(&url).send().await {
                Ok(resposta) => {
                    if let Ok(json) = resposta.json::<Value>().await {
                        let chave = moeda_string.replace("-", "");
                        let nome = json[&chave]["name"].as_str().unwrap_or("Desconhecido").to_string();
                        let valor = json[&chave]["bid"].as_str().unwrap_or("0.0").to_string();
                        
                        return Some((nome, valor));
                    }
                },
                Err(e) => println!("❌ Erro na API ({}): {}", moeda_string, e),
            }
            None
        });

        tarefas.push(tarefa);
    }

    let mut resultados = vec![];
    for tarefa in tarefas {
        if let Ok(Some(dado)) = tarefa.await {
            resultados.push(dado);
        }
    }

    // ==========================================
    // ETAPA 2: SALVAR EM CSV (Camada Bronze)
    // ==========================================
    println!("📄 Criando o arquivo CSV com os dados da API...");
    
    let nome_csv = "cotacoes_api.csv";
    let mut arquivo_csv = File::create(nome_csv).expect("❌ Erro ao criar o CSV");
    
    writeln!(arquivo_csv, "Moeda,Valor,DataHora").unwrap();
    
    let agora = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    for (nome, valor) in resultados {
        writeln!(arquivo_csv, "{},{},{}", nome, valor, agora).unwrap();
        println!("✅ Salvo no CSV: {} = R$ {}", nome, valor);
    }
    
    println!("\n🎉 Etapa 2 concluída! O arquivo '{}' foi gerado.", nome_csv);

    // ==========================================
    // ETAPA 3 E 4: LER O CSV E CONVERTER PARA PARQUET (Camada Ouro)
    // ==========================================
    println!("\n💾 Convertendo o CSV gerado para o formato Parquet...");

    let caminho_csv_string = nome_csv.to_string();
    
    tokio::task::spawn_blocking(move || {
        // Correção aplicada aqui: .as_str().into()
        let mut dataframe = LazyCsvReader::new(caminho_csv_string.as_str().into())
            .with_has_header(true)
            .finish()
            .expect("❌ Erro ao ler o CSV gerado")
            .collect()
            .expect("❌ Erro ao processar os dados do CSV");

        let nome_parquet = "cotacoes_api.parquet";
        let mut arquivo_parquet = File::create(nome_parquet).expect("❌ Erro ao criar Parquet");
        
        ParquetWriter::new(&mut arquivo_parquet)
            .finish(&mut dataframe)
            .expect("❌ Erro ao salvar o Parquet");

        println!("🎉 SUCESSO ABSOLUTO! O pipeline finalizou. O arquivo de alta performance '{}' está pronto.", nome_parquet);
    }).await.unwrap();
}