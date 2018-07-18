set -euxo pipefail

main() {
    case $TARGET in
        thumbv*-none-eabi*)
            rustup component list | grep 'rust-src.*installed' || \
                rustup component add rust-src
            ;;
    esac
}

main
