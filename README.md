# 🚀 Pipeline ETL em Rust (Arquitetura Medallion)

Este projeto é um pipeline de Engenharia de Dados de alta performance desenvolvido inteiramente em Rust. Ele executa a extração simultânea de cotações de moedas em tempo real e aplica os conceitos da **Arquitetura Medallion** (Camadas Bronze e Ouro).

⚠️ **Aviso de Transparência Acadêmica:** Este projeto foi desenvolvido com o auxílio de Inteligência Artificial (Assistente Gemini) para fins de estudo, estruturação de arquitetura e entrega de atividade universitária. A IA foi utilizada como ferramenta de pair-programming para acelerar o desenvolvimento do fluxo assíncrono e do processamento de dados.

## 🏗️ Arquitetura do Projeto

O fluxo de dados obedece às seguintes etapas (ETL):

1. **Extração Assíncrona (E):** Consome a API pública AwesomeAPI para buscar as cotações de Dólar, Euro, Bitcoin e Libra simultaneamente.
2. **Camada Bronze (Carga Raw - L):** Os dados brutos extraídos da web são imediatamente salvos em um arquivo `cotacoes_api.csv` contendo o timestamp (data e hora exatas) da execução.
3. **Camada Ouro (Transformação e Carga Final - T/L):** O arquivo CSV é lido em memória, processado e convertido para o formato `.parquet` (`cotacoes_api.parquet`), garantindo altíssima compressão e velocidade para futuras análises.

## 🛠️ Tecnologias Utilizadas

O "motor" deste projeto foi construído com as seguintes crates (bibliotecas) do ecossistema Rust:

* **[Tokio](https://tokio.rs/):** Runtime assíncrono para disparar múltiplas requisições web ao mesmo tempo sem travar a thread principal.
* **[Reqwest](https://docs.rs/reqwest/latest/reqwest/):** Cliente HTTP focado em facilidade e integração com JSON.
* **[Polars](https://pola.rs/):** Biblioteca de processamento de dados ultrarrápida (escrita em Rust), utilizada para ler o CSV e gerar o arquivo Parquet.
* **[Chrono](https://docs.rs/chrono/latest/chrono/):** Manipulação de datas e horas para o registro preciso (timestamp) na Camada Bronze.
* **[Serde](https://serde.rs/):** Serialização e desserialização robusta das respostas JSON da API.

## 🚀 Como Executar

Certifique-se de ter o [Rust e o Cargo](https://www.rust-lang.org/tools/install) instalados na sua máquina.

1. Clone este repositório.
2. Abra o terminal na raiz do projeto.
3. Execute o comando:

```bash
cargo run
