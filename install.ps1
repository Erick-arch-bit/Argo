# install.ps1 — Instalación del intérprete Argo en Windows
# Detecta la arquitectura, descarga el binario precompilado o compila desde fuente.
$ErrorActionPreference = "Stop"

$REPO = "Erick-arch-bit/Argo-Lang"
$VERSION = if ($env:VERSION) { $env:VERSION } else { "latest" }
$INSTALL_DIR = "$env:USERPROFILE\.local\bin"

# Detectar arquitectura
$arch = if ([Environment]::Is64BitOperatingSystem) {
    if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "arm64" } else { "amd64" }
} else {
    Write-Host "`n  [ERROR] Arquitectura de 32 bits no soportada." -ForegroundColor Red
    exit 1
}

$BINARIO = "argo-windows-$arch.exe"

Write-Host ""
Write-Host "  Argo - Instalacion para windows-$arch" -ForegroundColor Cyan
Write-Host ""

# Construir URL
if ($VERSION -eq "latest") {
    $url = "https://github.com/$REPO/releases/latest/download/$BINARIO"
    $sha_url = "https://github.com/$REPO/releases/latest/download/$BINARIO.sha256"
} else {
    $url = "https://github.com/$REPO/releases/download/$VERSION/$BINARIO"
    $sha_url = "https://github.com/$REPO/releases/download/$VERSION/$BINARIO.sha256"
}

# Crear directorio de instalación
if (-not (Test-Path $INSTALL_DIR)) {
    New-Item -ItemType Directory -Path $INSTALL_DIR -Force | Out-Null
}

$tmpdir = Join-Path $env:TEMP "argo-install-$(Get-Random)"
New-Item -ItemType Directory -Path $tmpdir -Force | Out-Null
$bin_path = Join-Path $tmpdir $BINARIO

try {
    Write-Host "  [download] Descargando $BINARIO..." -ForegroundColor DarkGray

    # Descargar binario
    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -Uri $url -OutFile $bin_path -UseBasicParsing
    } catch {
        Write-Host "  [ERROR] No se pudo descargar desde GitHub." -ForegroundColor Red
        Write-Host "  Compila desde fuente: cargo build --release" -ForegroundColor Yellow
        exit 1
    }

    # Verificar SHA256
    try {
        $expected = (Invoke-WebRequest -Uri $sha_url -UseBasicParsing).Content.Trim().Split(" ")[0]
        $actual = (Get-FileHash -Algorithm SHA256 $bin_path).Hash.ToLower()
        if ($expected -ne $actual) {
            Write-Host "  [ERROR] Verificacion SHA256 fallida." -ForegroundColor Red
            exit 1
        }
        Write-Host "  [ok] SHA256 verificada" -ForegroundColor DarkGray
    } catch {
        Write-Host "  [warn] No se pudo verificar SHA256, continuando..." -ForegroundColor Yellow
    }

    # Mover binario
    $dest = Join-Path $INSTALL_DIR "argo.exe"
    Copy-Item $bin_path $dest -Force
    Write-Host "  [install] Binario instalado en $dest" -ForegroundColor DarkYellow

} finally {
    Remove-Item -Recurse -Force $tmpdir -ErrorAction SilentlyContinue
}

# Agregar al PATH si no está
$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($currentPath -notlike "*$INSTALL_DIR*") {
    [Environment]::SetEnvironmentVariable("Path", "$INSTALL_DIR;$currentPath", "User")
    $env:Path = "$INSTALL_DIR;$env:Path"
    Write-Host "  [path] $INSTALL_DIR agregado al PATH del usuario" -ForegroundColor DarkYellow
    Write-Host "  Reinicia la terminal para aplicar los cambios." -ForegroundColor DarkGray
}

Write-Host ""
Write-Host "  Argo instalado exitosamente!" -ForegroundColor Green
Write-Host ""
