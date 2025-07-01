# Asset Relocator Tool

The Asset Relocator is a powerful tool built using the `etch_tsx` crate that helps you organize and internationalize assets in TypeScript/React projects. It automatically finds asset references in your TSX/TS files, moves them to a centralized location, and updates all file path references.

## Features

- 🔍 **Smart Asset Discovery**: Automatically finds assets referenced in TSX, TS, JSX, and JS files
- 📦 **Multiple Reference Types**: Detects assets in import statements, JSX attributes, and string literals
- 🎯 **Selective Processing**: Configurable file extensions and asset types
- 🏃 **Dry Run Mode**: Preview changes before applying them
- 📋 **Detailed Logging**: Verbose output to track the relocation process
- 🔄 **Copy or Move**: Choose whether to copy or move asset files
- 🎨 **Colored Output**: Easy-to-read console output with color coding

## Supported Asset Types

- **Images**: PNG, JPG, JPEG, GIF, SVG, WebP, ICO
- **Videos**: MP4, WebM, OGG
- **Audio**: MP3, WAV
- **Documents**: PDF, DOC, DOCX

## Installation

Build the tool from the project root:

```bash
cargo build --release --bin asset-relocator
```

## Usage

### Basic Usage

```bash
# Move assets from src/ to public/locales/en/
./target/release/asset-relocator \
  --source-dir /path/to/your/src \
  --target-dir /path/to/your/public/locales/en
```

### Advanced Usage

```bash
# Full example with all options
./target/release/asset-relocator \
  --source-dir /Users/hectorcrean/typescript/MEZ108_Radiesse_Training_Platform/src \
  --target-dir /Users/hectorcrean/typescript/MEZ108_Radiesse_Training_Platform/public/locales/en \
  --base-dir /Users/hectorcrean/typescript/MEZ108_Radiesse_Training_Platform \
  --extensions tsx,ts,jsx,js \
  --copy \
  --verbose \
  --dry-run
```

### Options

- `--source-dir, -s`: Source directory containing TSX files (required)
- `--target-dir, -t`: Target directory where assets should be moved (required)
- `--base-dir, -b`: Base directory for resolving relative paths (defaults to source-dir)
- `--dry-run, -d`: Preview changes without actually moving files or modifying code
- `--extensions, -e`: File extensions to process (default: "tsx,ts,jsx,js")
- `--copy, -c`: Copy assets instead of moving them
- `--verbose, -v`: Enable verbose output

## Example Workflow

### Before
```
my-project/
├── src/
│   ├── components/
│   │   ├── Header.tsx
│   │   └── images/
│   │       └── logo.png
│   ├── pages/
│   │   ├── Home.tsx
│   │   └── assets/
│   │       └── hero.jpg
│   └── data/
│       └── icons/
│           └── search.svg
└── public/
```

**Header.tsx**:
```typescript
import logo from './images/logo.png';

export function Header() {
  return <img src={logo} alt="Logo" />;
}

// Data file with asset references
export const pageData = {
  hero: {
    video: "intro.mp4",
    poster: "hero_thumb.jpg",
    tiles: {
      0: { image: "tile_01.jpg" },
      1: { image: "tile_02.jpg" },
    }
  }
};
```

### After Running Asset Relocator
```
my-project/
├── src/
│   ├── components/
│   │   └── Header.tsx
│   ├── pages/
│   │   └── Home.tsx
│   └── data/
└── public/
    └── locales/
        └── en/
            ├── logo.png
            ├── hero.jpg
            └── search.svg
```

**Header.tsx** (updated):
```typescript
import logo from '/public/locales/en/logo.png';

export function Header() {
  return <img src={logo} alt="Logo" />;
}

// Data file with updated asset references
export const pageData = {
  hero: {
    video: "/public/locales/en/intro.mp4",
    poster: "/public/locales/en/hero_thumb.jpg",
    tiles: {
      0: { image: "/public/locales/en/tile_01.jpg" },
      1: { image: "/public/locales/en/tile_02.jpg" },
    }
  }
};
```

## How It Works

The Asset Relocator uses a multi-phase approach:

### Phase 1: Discovery
- Scans all TypeScript/React files in the source directory
- Uses SWC (Speedy Web Compiler) to parse and analyze the code
- Identifies asset references in:
  - Import statements: `import image from './path/to/image.png'`
  - JSX attributes: `<img src="./image.jpg" />`, `<div data-bg-image="bg.png" />`
  - String literals: `const path = './assets/icon.svg'`
  - **Data objects**: `{ url: "video.mp4", poster: "thumb.jpg", image: "tile.png" }`
  - Template literals: `\`url('./background.jpg')\``

### Phase 2: Planning
- Analyzes discovered assets
- Creates a relocation plan mapping old paths to new paths
- Verifies that source files exist

### Phase 3: Directory Setup
- Creates the target directory structure if it doesn't exist

### Phase 4: Asset Relocation
- Moves or copies asset files to the target directory
- Preserves original filenames

### Phase 5: Code Updates
- Updates all file references in the source code
- Modifies import statements, JSX attributes, and string literals
- Maintains code formatting and structure

## Asset Visitor Architecture

The tool is built using a visitor pattern with the `AssetVisitor` struct that implements `VisitMut` from SWC:

```rust
pub struct AssetVisitor {
    current_file: PathBuf,
    base_dir: PathBuf,
    target_dir: PathBuf,
    assets: Vec<AssetReference>,
    path_mappings: HashMap<String, String>,
    asset_extensions: Vec<String>,
}
```

### Key Methods

- `visit_mut_import_decl()`: Processes import statements
- `visit_mut_jsx_attr()`: Handles JSX attributes like `src`, `href`, `data-*`, etc.
- `visit_mut_str()`: Analyzes all string literals for asset paths (including bare filenames)
- `visit_mut_tpl()`: Processes template literals
- `resolve_filename()`: Smart resolution of bare filenames using common asset directory patterns

## Error Handling

The tool provides detailed error messages for common issues:

- File parsing errors
- Missing source assets
- Permission issues
- Invalid path structures

## Performance

- Uses SWC for fast TypeScript/JavaScript parsing
- Parallel processing capabilities
- Efficient file I/O operations
- Memory-conscious asset discovery

## Special Features

### Smart Filename Resolution

The tool can now handle bare filenames (like `"tile_01.jpg"`) by trying multiple common asset directory patterns:

- Same directory as the file
- `./assets/`, `./assets/images/`, `./assets/videos/`, `./assets/audio/`
- `./images/`, `./videos/`, `./audio/`, `./media/`
- `./public/`, `./public/assets/`
- `../assets/`, `../public/assets/`

### Data File Support

Perfect for applications that use TSX files to store configuration data with asset references:

```typescript
export const moduleData = {
  sections: [{
    blocks: [{
      props: {
        url: "training_video.mp4",        // ✅ Detected
        poster: "video_poster.jpg",       // ✅ Detected
        tiles: {
          0: { image: "tile_01.jpg" },    // ✅ Detected
          1: { image: "tile_02.jpg" },    // ✅ Detected
        }
      }
    }]
  }]
};
```

All string values that match asset file extensions will be detected and relocated.

## Integration with Internationalization

This tool is particularly useful for setting up internationalized asset structures:

```bash
# Move assets to English locale
./asset-relocator -s src -t public/locales/en

# Copy the same assets to other locales
cp -r public/locales/en/* public/locales/fr/
cp -r public/locales/en/* public/locales/es/
```

## Troubleshooting

### Common Issues

1. **Permission Denied**: Ensure you have write permissions to both source and target directories
2. **File Not Found**: Check that asset files actually exist at the referenced paths
3. **Parse Errors**: Verify that TypeScript files are syntactically correct

### Debug Mode

Use `--verbose` flag to see detailed information about the relocation process:

```bash
./asset-relocator -s src -t dist/assets --verbose --dry-run
```

## Contributing

The Asset Relocator is part of the larger `etch` project. To contribute:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Submit a pull request

## License

This tool is part of the etch project and follows the same licensing terms. 