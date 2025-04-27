# Maintainer: Marcos da Silva <marcossl10@hotmail.com>
# Contributor:

pkgname=rust-music-player-lite # Nome do pacote
_pkgbasename=rust-music-player-lite-alfa # Nome base do repositório para o diretório fonte
pkgver=1.1.0 # Versão do pacote
pkgrel=1 # Revisão do PKGBUILD
pkgdesc="Um player de música simples e leve feito em Rust com egui." # Descrição curta
arch=('x86_64') # Arquitetura
url="https://github.com/marcossl10/rust-music-player-lite-alfa" # URL do projeto ATUALIZADO
license=('MIT') # Licença (Certifique-se que o arquivo LICENSE existe no repo)
makedepends=('rustup') # Dependências de compilação
depends=('alsa-lib' 'libxcb' 'libxkbcommon' 'openssl') # Dependências de execução
source=("$_pkgbasename-$pkgver.tar.gz::$url/archive/refs/tags/v$pkgver.tar.gz") # Fonte ATUALIZADA
# SUBSTITUA 'COLOQUE_O_HASH_AQUI' pelo hash calculado!
sha256sums=('COLOQUE_O_HASH_AQUI')

# Função para compilar o código fonte
build() {
    # Navega para o diretório do código fonte descompactado
    # O nome do diretório geralmente é <nome-repo>-<versao>
    cd "$srcdir/$_pkgbasename-$pkgver"

    # Garante que o toolchain Rust esteja configurado
    rustup default stable
    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR=target # Compila dentro da pasta do build

    # Compila o projeto em modo release
    cargo build --release --locked
}

# Função para instalar os arquivos no diretório temporário do pacote ($pkgdir)
package() {
    cd "$srcdir/$_pkgbasename-$pkgver"

    # Instala o executável principal (use o nome final do pacote)
    install -Dm755 "target/release/music-player-lite" "$pkgdir/usr/bin/$pkgname"

    # Instala o arquivo de licença (Assume que existe um arquivo LICENSE na raiz do repo)
    install -Dm644 LICENSE "$pkgdir/usr/share/licenses/$pkgname/LICENSE"

    # Opcional: Instala o ícone (se existir 'assets/icon.png' no repo)
    # install -Dm644 "assets/icon.png" "$pkgdir/usr/share/icons/hicolor/64x64/apps/$pkgname.png"

    # Opcional: Instala o arquivo .desktop (se existir 'assets/rust-music-player-lite.desktop' no repo)
    # install -Dm644 "assets/rust-music-player-lite.desktop" "$pkgdir/usr/share/applications/$pkgname.desktop"
}

# Opcional: Função check()
#check() {
#    cd "$srcdir/$_pkgbasename-$pkgver"
#    cargo test
#}
