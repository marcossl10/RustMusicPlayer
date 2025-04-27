# Rust Music Player Lite (Alfa 1.1)

Um player de música simples e leve escrito em Rust, utilizando as bibliotecas `egui` para a interface gráfica e `rodio`/`symphonia` para a reprodução de áudio.

*(Opcional: Adicione um screenshot aqui se desejar)*
<!-- !Screenshot -->

## Funcionalidades

*   **Reprodução de Áudio:** Toca arquivos de áudio nos formatos suportados por Symphonia (MP3, WAV, FLAC, Ogg Vorbis, etc.).
*   **Controles Básicos:** Play, Pause, Stop, Próxima Faixa, Faixa Anterior.
*   **Barra de Progresso:** Visualiza e permite buscar (seek) diferentes partes da música.
*   **Controle de Volume:** Ajusta o volume da reprodução.
*   **Gerenciamento de Playlist:**
    *   Adicionar múltiplos arquivos de áudio.
    *   Remover faixas selecionadas.
    *   Limpar toda a playlist.
    *   Seleção e reprodução de faixas clicando na lista.
*   **Modos de Reprodução:**
    *   Shuffle (Ordem Aleatória).
    *   Repeat (Desligado, Repetir Playlist, Repetir Faixa Atual).
*   **Persistência:** Salva o estado da playlist, volume e modos de reprodução ao fechar.
*   **Interface Simples:** Criada com `egui`.
*   **Janela "Sobre":** Exibe informações sobre o player e o desenvolvedor.

## Instalação (Arch Linux)

Este pacote pode ser construído e instalado usando o `PKGBUILD` fornecido.

1.  **Instale as dependências de compilação e execução:**
    ```bash
    sudo pacman -Syu --needed rustup alsa-lib libxcb libxkbcommon openssl git base-devel
    ```
    *(O `base-devel` inclui ferramentas como `makepkg`)*

2.  **Clone o repositório:**
    ```bash
    git clone https://github.com/marcossl10/rust-music-player-lite-alfa.git
    cd rust-music-player-lite-alfa
    ```

3.  **Construa e instale o pacote:**
    ```bash
    makepkg -si
    ```
    *(O comando `-s` instala as dependências listadas no `PKGBUILD` e o `-i` instala o pacote após a construção)*

Após a instalação, o player estará disponível no seu menu de aplicativos (se o arquivo `.desktop` for incluído no `PKGBUILD`) ou pode ser executado pelo terminal com o comando `rust-music-player-lite`.

## Compilação Manual (Desenvolvimento)

Se preferir compilar manualmente:

1.  **Instale o Rust:** Siga as instruções em rustup.rs.
2.  **Instale as dependências de sistema (Arch Linux):**
    ```bash
    sudo pacman -Syu --needed alsa-lib libxcb libxkbcommon openssl
    ```
3.  **Clone o repositório:**
    ```bash
    git clone https://github.com/marcossl10/rust-music-player-lite-alfa.git
    cd rust-music-player-lite-alfa
    ```
4.  **Compile e execute (modo debug):**
    ```bash
    cargo run
    ```
5.  **Compile (modo release):**
    ```bash
    cargo build --release
    ```
    O executável estará em `target/release/music-player-lite`.

## Como Usar

1.  Execute o programa (`rust-music-player-lite` ou via menu).
2.  Clique em "**➕ Add**" para adicionar arquivos de áudio à playlist.
3.  Selecione uma música na lista e clique em "**Play ▶**" ou clique duas vezes na música na lista.
4.  Use os botões de controle para pausar, parar, pular faixas, etc.
5.  Ajuste o volume com o slider.
6.  Use os botões "**🔀**" e "**🔁**" para ativar os modos Shuffle e Repeat.
7.  Use os botões "**➖ Remove**" e "**🗑️ Clear**" para gerenciar a playlist.
8.  Acesse o menu "**Ajuda**" -> "**Sobre...**" para informações do desenvolvedor.

## Tecnologias Utilizadas

*   **Linguagem:** Rust
*   **Interface Gráfica:** egui (via eframe)
*   **Reprodução de Áudio (Backend):** rodio
*   **Decodificação de Áudio:** symphonia
*   **Leitura de Metadados (Duração):** lofty
*   **Seleção de Arquivos:** rfd (Rust File Dialog)
*   **Comunicação entre Threads:** crossbeam-channel

## Licença

Este projeto é licenciado sob a Licença MIT. Veja o arquivo `LICENSE` para mais detalhes.

## Contato

*   **Desenvolvedor:** Marcos da Silva
*   **Email:** marcossl10@hotmail.com
*   **Pix (Café):** 83980601072

---


# Rust Music Player Lite (Alpha 1.1)

A simple and lightweight music player written in Rust, using the `egui` library for the graphical interface and `rodio`/`symphonia` for audio playback.

*(Optional: Add a screenshot here if you wish)*
<!-- !Screenshot -->

## Features

*   **Audio Playback:** Plays audio files in formats supported by Symphonia (MP3, WAV, FLAC, Ogg Vorbis, etc.).
*   **Basic Controls:** Play, Pause, Stop, Next Track, Previous Track.
*   **Progress Bar:** Visualizes and allows seeking through different parts of the music.
*   **Volume Control:** Adjusts the playback volume.
*   **Playlist Management:**
    *   Add multiple audio files.
    *   Remove selected tracks.
    *   Clear the entire playlist.
    *   Select and play tracks by clicking on the list.
*   **Playback Modes:**
    *   Shuffle (Random Order).
    *   Repeat (Off, Repeat Playlist, Repeat Current Track).
*   **Persistence:** Saves the playlist state, volume, and playback modes upon closing.
*   **Simple Interface:** Created with `egui`.
*   **"About" Window:** Displays information about the player and the developer.

## Installation (Arch Linux)

This package can be built and installed using the provided `PKGBUILD`.

1.  **Install build and runtime dependencies:**
    ```bash
    sudo pacman -Syu --needed rustup alsa-lib libxcb libxkbcommon openssl git base-devel
    ```
    *(The `base-devel` group includes tools like `makepkg`)*

2.  **Clone the repository:**
    ```bash
    git clone https://github.com/marcossl10/rust-music-player-lite-alfa.git
    cd rust-music-player-lite-alfa
    ```

3.  **Build and install the package:**
    ```bash
    makepkg -si
    ```
    *(The `-s` command installs dependencies listed in the `PKGBUILD`, and `-i` installs the package after building)*

After installation, the player will be available in your application menu (if the `.desktop` file is included in the `PKGBUILD`) or can be run from the terminal with the command `rust-music-player-lite`.

## Manual Compilation (Development)

If you prefer to compile manually:

1.  **Install Rust:** Follow the instructions at rustup.rs.
2.  **Install system dependencies (Arch Linux):**
    ```bash
    sudo pacman -Syu --needed alsa-lib libxcb libxkbcommon openssl
    ```
3.  **Clone the repository:**
    ```bash
    git clone https://github.com/marcossl10/rust-music-player-lite-alfa.git
    cd rust-music-player-lite-alfa
    ```
4.  **Compile and run (debug mode):**
    ```bash
    cargo run
    ```
5.  **Compile (release mode):**
    ```bash
    cargo build --release
    ```
    The executable will be located at `target/release/music-player-lite`.

## How to Use

1.  Run the program (`rust-music-player-lite` or via the menu).
2.  Click "**➕ Add**" to add audio files to the playlist.
3.  Select a song from the list and click "**Play ▶**" or double-click the song in the list.
4.  Use the control buttons to pause, stop, skip tracks, etc.
5.  Adjust the volume using the slider.
6.  Use the "**🔀**" and "**🔁**" buttons to activate Shuffle and Repeat modes.
7.  Use the "**➖ Remove**" and "**🗑️ Clear**" buttons to manage the playlist.
8.  Access the "**Help**" -> "**About...**" menu for developer information.

## Technologies Used

*   **Language:** Rust
*   **GUI:** egui (via eframe)
*   **Audio Backend:** rodio
*   **Audio Decoding:** symphonia
*   **Metadata Reading (Duration):** lofty
*   **File Dialogs:** rfd (Rust File Dialog)
*   **Thread Communication:** crossbeam-channel

## License

This project is licensed under the MIT License. See the `LICENSE` file for more details.

## Contact

*   **Developer:** Marcos da Silva
*   **Email:** marcossl10@hotmail.com
*   **Pix (Coffee):** 83980601072

---
