# Maintainer: Your Name <your@email.com>
pkgname=zemon-bin
pkgver=0.2.1
pkgrel=1
pkgdesc="A terminal system monitor written in Rust"
arch=('x86_64')
url="https://github.com/yourusername/zemon"
license=('MIT')
depends=('glibc')
source=("https://github.com/yourusername/zemon/releases/download/v${pkgver}/zemon-linux-x86_64")
sha256sums=('SKIP') # You'll need to replace this with actual checksum after first release

package() {
  install -Dm755 "$srcdir/zemon-linux-x86_64" "$pkgdir/usr/bin/zemon"
}
