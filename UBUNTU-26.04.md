# Desarrollo y compilacion en Ubuntu 26.04

vtt-rol es una aplicacion de escritorio Tauri 2. Esta guia instala las
dependencias necesarias para ejecutarla en modo desarrollo y generar un paquete
`.deb` nativo para Ubuntu 26.04.

## Requisitos

- Ubuntu 26.04 de 64 bits.
- Bun para instalar dependencias y ejecutar el frontend.
- Rust estable, instalado con rustup, para compilar el backend Tauri.
- Bibliotecas de desarrollo de GTK, WebKitGTK y las herramientas de compilacion
  indicadas abajo.

## 1. Dependencias del sistema

En una terminal, instala las bibliotecas y herramientas requeridas por Tauri:

```bash
sudo apt update
sudo apt install --yes \
  build-essential \
  curl \
  file \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libssl-dev \
  libwebkit2gtk-4.1-dev \
  libxdo-dev \
  wget
```

## 2. Bun y Rust

Instala Bun siguiendo las instrucciones oficiales de
[bun.sh](https://bun.sh/docs/installation). Comprueba que ambos comandos esten
disponibles en una terminal nueva:

```bash
bun --version
```

Instala Rust estable mediante rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"
rustup toolchain install stable
rustup default stable
rustc --version
cargo --version
```

## 3. Preparar y ejecutar el proyecto

Desde la raiz del repositorio:

```bash
bun install --frozen-lockfile
bun run tauri dev
```

`bun install --frozen-lockfile` instala las versiones guardadas en `bun.lock`.
El comando de desarrollo compila el frontend y el backend y abre la ventana de
la aplicacion.

Para validar el backend:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

## 4. Crear un paquete de Ubuntu

Genera un paquete Debian para la arquitectura del equipo:

```bash
bun run tauri build --bundles deb
```

El paquete resultante se guarda en
`src-tauri/target/release/bundle/deb/`. Para instalarlo, usa el nombre del
archivo `.deb` generado:

```bash
sudo apt install ./src-tauri/target/release/bundle/deb/NOMBRE_DEL_PAQUETE.deb
```

## Integracion continua

El flujo `.github/workflows/ubuntu-26.04.yml` ejecuta las pruebas Rust y genera
el paquete `.deb` en un runner Ubuntu 26.04 cada vez que se actualiza `main` o
se abre un pull request hacia esa rama.
