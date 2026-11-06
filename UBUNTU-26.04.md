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
  pkg-config \
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

Si ejecutas el proyecto desde la terminal integrada de VS Code instalado como
Snap y GTK muestra un error de esquema de GNOME, inicia la aplicacion con un
entorno limpio para no heredar las variables de Snap:

```bash
env -i \
  HOME="$HOME" \
  USER="$USER" \
  LOGNAME="$LOGNAME" \
  PATH="$HOME/.bun/bin:$HOME/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin" \
  DISPLAY="${DISPLAY:-}" \
  WAYLAND_DISPLAY="${WAYLAND_DISPLAY:-}" \
  XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-}" \
  DBUS_SESSION_BUS_ADDRESS="${DBUS_SESSION_BUS_ADDRESS:-}" \
  XDG_DATA_DIRS="${XDG_DATA_DIRS_VSCODE_SNAP_ORIG:-/usr/share/ubuntu:/usr/share/gnome:/usr/local/share:/usr/share:/var/lib/snapd/desktop}" \
  bun run tauri dev
```

La aplicacion guarda las imagenes de las fichas en su directorio de datos de
usuario. El protocolo de archivos de Tauri tiene acceso limitado a
`$APPDATA/**` para poder mostrarlas sin dar acceso al resto del sistema.

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

## Desinstalar lo instalado para las pruebas

Desde la raiz del repositorio, elimina los artefactos generados durante las
pruebas:

```bash
rm -rf node_modules build src-tauri/target
```

Si instalaste el paquete `.deb`, desinstala la aplicacion:

```bash
sudo apt remove vtt-rol
```

Si las dependencias de desarrollo se instalaron exclusivamente para este
proyecto y no las usan otras aplicaciones, puedes quitarlas y limpiar
dependencias que hayan quedado sin uso:

```bash
sudo apt remove --purge \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  libssl-dev \
  libwebkit2gtk-4.1-dev \
  libxdo-dev
sudo apt autoremove --purge
```

Revisa los paquetes que `apt` propone antes de confirmar. No se incluyen
`build-essential`, `curl`, `file`, `pkg-config` ni `wget`, porque son
herramientas de uso general y pueden ser necesarias para otros proyectos.

Si instalaste Bun o Rust exclusivamente para estas pruebas, desinstala Bun
quitando `~/.bun` y las entradas de `PATH` que su instalador agrego a tu
configuracion de shell. Desinstala Rust con:

```bash
rustup self uninstall
```

No ejecutes estos pasos si Bun o Rust ya se utilizaban para otros proyectos.
