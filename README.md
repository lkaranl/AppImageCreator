# AppImageCreator

Gerador gráfico de AppImages escrito em Rust com GTK4 e libadwaita. Permite montar rapidamente um pacote AppImage a partir de um binário já compilado, escolhendo ícone, metadados e destino final com uma interface moderna.

## Recursos
- Interface GTK4/libadwaita com organização em grupos de preferências.
- Seleção guiada do binário executável, ícone e pasta de saída.
- Campos para nome, comando `Exec`, categorias (com seleção por checkboxes), versão, descrição, autor, licença e website.
- Conversão automática do ícone para PNG (aceita PNG, JPG, ICO, BMP, etc).
- Geração assíncrona do AppImage com indicador visual (texto e barra de progresso animada).
- Feedback ao concluir via toast (sucesso ou erro).

## Pré-requisitos
- Rust 1.75+ (com `cargo`).
- Dependências de desenvolvimento do GTK4/libadwaita instaladas no sistema.
- `cargo-appimage` instalado globalmente:  
  ```bash
  cargo install cargo-appimage
  ```

## Instalação
Clone o repositório e instale as dependências:
```bash
git clone https://github.com/lkaranl/AppImageCreator.git
cd AppImageCreator
cargo fetch
```

## Uso
1. Compile e execute:
   ```bash
   cargo run
   ```
2. Na interface:
   - Escolha o binário do seu aplicativo.
   - Defina o ícone (qualquer formato suportado).
   - Preencha os metadados obrigatórios.
   - Selecione a pasta onde o AppImage será salvo.
   - Clique em **Gerar AppImage** e aguarde o indicativo de progresso.
3. Ao término, o arquivo `.AppImage` será criado na pasta escolhida.

## Estrutura principal
- `src/main.rs`: interface gráfica, estados e validações.
- `src/appimage.rs`: rotina de geração, criação de metadados, conversão de ícones e chamada ao `cargo appimage`.

## Desenvolvimento
Rodar clippy e testes:
```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

## Licença
Este projeto é disponibilizado sob a licença GPL-3.0-or-later. Consulte o arquivo `LICENSE` (se existente) para mais detalhes ou ajuste conforme necessário.