# Maintainer: Marcos da Silva <marcossl10@hotmail.com>
# Contributor:

pkgname=rust-music-player-lite # Nome do pacote
_pkgbasename=RustMusicPlayer # Nome base do repositório para o diretório fonte (CORRIGIDO)
pkgver=1.1.0 # Versão do pacote
pkgrel=1 # Revisão do PKGBUILD
pkgdesc="Um player de música simples e leve feito em Rust com egui." # Descrição curta
arch=('x86_64') # Arquitetura
url="https://github.com/marcossl10/RustMusicPlayer.git" # URL do projeto (CORRIGIDO - sem ponto final)
license=('MIT') # Licença (Certifique-se que o arquivo LICENSE existe no repo)
makedepends=('rustup' 'cargo') # Dependências de compilação (Adicionado 'cargo' explicitamente)
depends=('alsa-lib' 'libxcb' 'libxkbcommon' 'openssl') # Dependências de execução

# Fonte corrigida para usar o nome do repositório correto e a variável _pkgbasename
# ATENÇÃO: Certifique-se que a tag v1.1.0 existe no repositório RustMusicPlayer!
 source=("$_pkgbasename::git+https://github.com/marcossl10/$_pkgbasename.git#branch=master")


# Use 'SKIP' temporariamente. Rode 'updpkgsums' após o primeiro download falhar para gerar o hash correto.
sha256sums=('SKIP')

# Função para compilar o código fonte
build() {
    # Navega para o diretório do código fonte descompactado (CORRIGIDO)
    cd "$srcdir/$_pkgbasename"

    # Garante que o toolchain Rust esteja configurado
    # Nota: 'rustup default stable' pode exigir interação, geralmente não é necessário dentro do build.
    # O sistema de build do Arch geralmente já tem o Rust configurado.
    # Apenas exportar as variáveis costuma ser suficiente.
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target # Compila dentro da pasta do build

    # Compila o projeto em modo release
    cargo build --release --locked
}

# Função para instalar os arquivos no diretório temporário do pacote ($pkgdir)
package() {
    # Navega para o diretório do código fonte descompactado (CORRIGIDO)
    cd "$srcdir/$_pkgbasename"

    # Instala o executável principal (CORRIGIDO - usando o nome real do executável)
    install -Dm755 "target/release/RustMusicPlayer" "$pkgdir/usr/bin/$pkgname"

    # Instala o arquivo de licença (Assume que existe um arquivo LICENSE na raiz do repo)
    install -Dm644 LICENSE.txt "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
    install -Dm644 "$pkgname.desktop" "$pkgdir/usr/share/applications/$pkgname.desktop"
    # Opcional: Instala o ícone (se existir 'assets/icon.png' no repo)
    # Descomente e ajuste o caminho se necessário
    # install -Dm644 "assets/icon.png" "$pkgdir/usr/share/icons/hicolor/64x64/apps/$pkgname.png"

    # Opcional: Instala o arquivo .desktop (se existir 'assets/rust-music-player-lite.desktop' no repo)
    # Descomente e ajuste o caminho se necessário
    # install -Dm644 "assets/$pkgname.desktop" "$pkgdir/usr/share/applications/$pkgname.desktop"
}

# Opcional: Função check() para rodar testes
#check() {
#    cd "$srcdir/$_pkgbasename"
#    export RUSTUP_TOOLCHAIN=stable # Garante o toolchain para testes também
#    cargo test --locked
#}