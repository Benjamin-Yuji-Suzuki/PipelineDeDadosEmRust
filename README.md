# 🚀 Pipeline ETL em Rust (Arquitetura Medallion)

Este projeto é um pipeline de Engenharia de Dados de alta performance desenvolvido inteiramente em Rust. Ele executa a extração de dados de um e-commerce simulado e aplica os conceitos completos da **Arquitetura Medallion** (Camadas Bronze, Silver e Gold) para realizar processos de ETL (Extract, Transform, Load).

⚠️ **Aviso de Transparência Acadêmica:** Este projeto foi desenvolvido com o auxílio de Inteligência Artificial (Assistente Gemini) para fins de estudo, estruturação de arquitetura e entrega de atividade universitária. A IA foi utilizada como ferramenta de pair-programming para acelerar o desenvolvimento do fluxo assíncrono e do processamento de dados.

## 🏗️ Arquitetura do Projeto

O fluxo de dados atende aos requisitos de limpeza, padronização e estruturação, obedecendo às seguintes etapas do ETL:

1. **Extração Assíncrona (Extract):** Consome a API pública Fake Store para buscar os dados brutos de produtos (ID, título, categoria, preço em dólar e avaliação).
2. **Camada Bronze (Raw Data):** Os dados extraídos do formato JSON são imediatamente estruturados e salvos em um arquivo local `bronze_produtos.csv`. Durante esta etapa, já ocorre uma engenharia defensiva para remover quebras de linha e vírgulas dos títulos.
3. **Camada Silver (Transformação - Transform):** O arquivo CSV bruto é carregado em memória usando Polars, onde ocorrem as seguintes regras de negócio:
   * **Tipagem Rigorosa (Casting):** Garantia de tipos corretos (Int64 e Float64) para evitar falhas de inserção no banco.
   * **Padronização:** Conversão de todos os textos da coluna de categoria para MAIÚSCULO.
   * **Cálculo:** Criação de uma nova coluna convertendo o preço em Dólar para Real (`preco_brl`), multiplicando por 5.0.
   * **Limpeza/Filtro:** Remoção de produtos com nota de avaliação inferior ou igual a 3.0.
4. **Camada Gold (Carregamento - Load):** Os dados limpos e transformados são inseridos de forma automatizada em um banco de dados relacional SQLite (`gold_ecommerce.db`), na tabela estruturada `produtos_gold`.

## 🛠️ Tecnologias Utilizadas

O "motor" deste projeto foi construído com as seguintes crates (bibliotecas) do ecossistema Rust:

* **[Tokio](https://tokio.rs/):** Runtime assíncrono para disparar a requisição web sem travar a thread principal.
* **[Reqwest](https://docs.rs/reqwest/latest/reqwest/):** Cliente HTTP focado em facilidade e integração com JSON.
* **[Polars](https://pola.rs/):** Biblioteca de processamento de dados ultrarrápida (escrita em Rust), utilizada para ler o CSV, filtrar e transformar os dados na Camada Silver.
* **[Rusqlite](https://docs.rs/rusqlite/latest/rusqlite/):** Interface ergonômica para interagir com o banco de dados relacional SQLite, executando a criação de tabelas e inserção dos dados (Camada Gold).
* **[Chrono](https://docs.rs/chrono/latest/chrono/):** Manipulação de datas e horas para o registro preciso (timestamp) de conclusão da pipeline.
* **[Serde](https://serde.rs/):** Serialização e desserialização robusta das respostas JSON da API.

## 🚀 Como Executar

Certifique-se de ter o [Rust e o Cargo](https://www.rust-lang.org/tools/install) instalados na sua máquina.

1. Clone este repositório.
2. Abra o terminal na raiz do projeto.
3. Execute o comando:

```bash
cargo run