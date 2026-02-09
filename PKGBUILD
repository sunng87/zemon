# Maintainer: Ning Sun <n@sunng.info>
pkgname=zemon-bin
pkgver=0.3.0
pkgrel=1
pkgdesc="A terminal system monitor for zellij written in Rust"
arch=('x86_64')
url="https://github.com/sunng87/zemon"
provides=('zemon')
license=('MIT')
depends=('glibc')
makedepends=('patchelf')
source=("$pkgname-$pkgver::https://github.com/sunng87/zemon/releases/download/v${pkgver}/zemon-linux-x86_64")
sha256sums=('SKIP') # You'll need to replace this with actual checksum after first release

package() {
  patchelf --set-interpreter /usr/lib64/ld-linux-x86-64.so.2 "$srcdir/$pkgname-$pkgver"
  install -Dm755 "$srcdir/$pkgname-$pkgver" "$pkgdir/usr/bin/zemon"
}
