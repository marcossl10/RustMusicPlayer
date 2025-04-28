# Maintainer: Marcos da Silva <marcossl10@hotmail.com>
# Contributor:

pkgname=rust-music-player-lite
_pkgbasename=RustMusicPlayer
pkgver=1.1.0
pkgrel=1
pkgdesc="Um player de música simples e leve feito em Rust com egui."
arch=('x86_64')
url="https://github.com/marcossl10/RustMusicPlayer.git"
license=('MIT')
makedepends=('rustup' 'cargo')
depends=('alsa-lib' 'libxcb' 'libxkbcommon' 'openssl')

source=("$_pkgbasename::git+https://github.com/marcossl10/$_pkgbasename.git#branch=master")
sha256sums=('SKIP')

build() {
    cd "$srcdir/$_pkgbasename"
    export CARGO_TARGET_DIR="$srcdir/target"
    cargo build --release --locked
}

package() {
    cd "$srcdir/$_pkgbasename"
    echo "Instalando o executável..."
    install -Dm755 "target/release/RustMusicPlayer" "$pkgdir/usr/bin/$pkgname"
    echo "Instalando a licença..."
    install -Dm644 "LICENCE.txt" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
    echo "Instalando o arquivo .desktop..."
    install -Dm644 "$pkgname.desktop" "$pkgdir/usr/share/applications/$pkgname.desktop"

    echo "Instalando ícones..."
    echo "  16x16:"
    ls -l "icons/rust-music-player-lite-16x16.png"
    install -Dm644 "icons/rust-music-player-lite-16x16.png" "$pkgdir/usr/share/icons/hicolor/16x16/apps/$pkgname.png"
    echo "  32x32:"
    ls -l "icons/rust-music-player-lite-32x32.png"
    install -Dm644 "icons/rust-music-player-lite-32x32.png" "$pkgdir/usr/share/icons/hicolor/32x32/apps/$pkgname.png"
    echo "  48x48:"
    ls -l "icons/rust-music-player-lite-48x48.png"
    install -Dm644 "icons/rust-music-player-lite-48x48.png" "$pkgdir/usr/share/icons/hicolor/48x48/apps/$pkgname.png"
    echo "  64x64:"
    ls -l "icons/rust-music-player-lite-64x64.png"
    install -Dm644 "icons/rust-music-player-lite-64x64.png" "$pkgdir/usr/share/icons/hicolor/64x64/apps/$pkgname.png"
    echo "  128x128:"
    ls -l "icons/rust-music-player-lite-128x128.png"
    install -Dm644 "icons/rust-music-player-lite-128x128.png" "$pkgdir/usr/share/icons/hicolor/128x128/apps/$pkgname.png"
    echo "  256x256:"
    ls -l "icons/rust-music-player-lite-256x256.png"
    install -Dm644 "icons/rust-music-player-lite-256x256.png" "$pkgdir/usr/share/icons/hicolor/256x256/apps/$pkgname.png"
}