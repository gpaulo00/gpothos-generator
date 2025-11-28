# 🚀 Prisma Pothos Generator

**Generador de código GraphQL Pothos a partir de esquemas Prisma**, compatible con TypeGraphQL y diseñado para acelerar el desarrollo de APIs GraphQL type-safe.

## 📋 Descripción

`gpothos-generator` es una herramienta CLI escrita en Rust que genera automáticamente código [Pothos GraphQL](https://pothos-graphql.dev/) a partir de tu esquema Prisma. Genera tipos, inputs, filtros, enums y resolvers completos (queries y mutations) listos para usar.

La idea del proyecto es poder migrar un proyecto que ya utiliza TypeGraphQL a Pothos GraphQL, manteniendo la funcionalidad existente y agregando la ventaja de ser más rápido.

### ✨ Características Principales

- 🔄 **Generación automática** de tipos GraphQL desde Prisma
- 🎯 **Type-safe** con soporte completo para TypeScript
- 🔍 **Detección inteligente** de resolvers manuales para evitar duplicados
- ⚡ **Alto rendimiento** gracias a Rust
- 🛠️ **Configurable** mediante archivo `.gpothosrc.json`
- 📦 **Compatible** con proyectos TypeGraphQL existentes
- 🔌 **Integración** como generador de Prisma o CLI standalone

## 📦 Instalación

### Requisitos Previos

- [Rust](https://www.rust-lang.org/tools/install) (1.70+)
- [Cargo](https://doc.rust-lang.org/cargo/)
- Node.js y npm (para el proyecto GraphQL)

### Instalación Local

```bash
# Clonar el repositorio
git clone <repository-url>
cd prisma-pothos-generator

# Compilar el proyecto
cargo build --release

# El binario estará en target/release/gpothos-generator
```

### Instalación como Dependencia npm

```bash
npm install
```

El script `postinstall` compilará automáticamente el binario de Rust.

## 🚀 Uso

### Modo CLI

```bash
# Uso básico
./target/release/gpothos-generator -s ./prisma/schema.prisma -o ./src/generated

# Con opciones personalizadas
./target/release/gpothos-generator \
  --schema ./path/to/schema.prisma \
  --output ./src/graphql/generated
```

#### Opciones CLI

| Opción | Alias | Default | Descripción |
|--------|-------|---------|-------------|
| `--schema` | `-s` | `./prisma/schema.prisma` | Ruta al archivo schema de Prisma |
| `--output` | `-o` | `./src/generated` | Directorio de salida para archivos generados |
| `--prisma-generator` | - | `false` | Ejecutar como generador de Prisma (lee DMMF desde stdin) |

### Modo Generador de Prisma

**NOTA**: Aún no esta funcionando el generador de Prisma, funciona muy lento al parecer.

Agrega el generador a tu `schema.prisma`:

```prisma
generator pothos {
  provider = "gpothos-generator"
  output   = "../src/generated"
}
```

Luego ejecuta:

```bash
npx prisma generate
```

## ⚙️ Configuración

### Archivo `.gpothosrc.json`

Crea un archivo `.gpothosrc.json` en la raíz de tu proyecto para configurar el comportamiento del generador:

```json
{
  "autoScan": true,
  "scanDirs": ["src/types", "src/pothos", "src/graphql"],
  "verbose": false
}
```

#### Opciones de Configuración

| Campo | Tipo | Default | Descripción |
|-------|------|---------|-------------|
| `autoScan` | `boolean` | `true` | Habilita el escaneo automático de resolvers manuales |
| `scanDirs` | `string[]` | `[]` | Directorios a escanear para detectar resolvers manuales |
| `verbose` | `boolean` | `false` | Muestra logs detallados durante la generación |

## 🔍 Detección de Resolvers Manuales

Una de las características más poderosas del generador es la **detección automática de resolvers manuales**, que evita la generación de código duplicado.

### ¿Cómo Funciona?

1. El generador escanea los directorios especificados en `scanDirs`
2. Busca patrones de `builder.queryField()` y `builder.mutationField()`
3. Excluye automáticamente esos resolvers de la generación

## 🛠️ Desarrollo

### Compilar

```bash
# Debug build
cargo build

# Release build (optimizado)
cargo build --release
```

### Ejecutar Tests

```bash
cargo test
```

### Linting

```bash
cargo clippy
```

### Formatear Código

```bash
cargo fmt
```

## 📚 Documentación Adicional

- [Pothos GraphQL](https://pothos-graphql.dev/) - Documentación oficial de Pothos
- [Prisma](https://www.prisma.io/) - Documentación oficial de Prisma

## 📝 Licencia

Este proyecto está bajo la licencia MIT. Ver archivo `LICENSE` para más detalles.
