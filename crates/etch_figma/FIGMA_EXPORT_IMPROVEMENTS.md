# Figma Export Pipeline Improvements

This document summarizes the enhancements made to the etch Figma export pipeline, inspired by Figma Make's approach.

## Implemented Features

### 1. Path Registry System ✅

**File:** `src/tsx/path_registry.rs`

Extracts complex SVG paths to a separate TypeScript file, similar to Figma Make's approach.

```typescript
// Generated svg-paths.ts file
export default {
  p2cf2ca00: "M10,10 L90,90...",
  p260a4790: "M0,0 C...",
  // ... more paths
};
```

**Usage in components:**
```tsx
import svgPaths from "./svg-paths";

<path d={svgPaths.p2cf2ca00} fill="url(#paint0_linear_1_405)" />
```

**Benefits:**
- Reduces component file size
- Enables better code splitting
- Improves readability
- Deduplicates repeated paths

### 2. CSS Variable Theming Support ✅

**File:** `src/tsx/paint.rs`

Added functions to generate CSS variables with fallbacks for theming:

```rust
pub fn fill_to_svg_attr_with_var(fills: &[Paint], var_index: usize) 
    -> Option<(&'static str, String)>

pub fn stroke_to_svg_attrs_with_var(
    strokes: &Option<Vec<Paint>>, 
    stroke_weight: Option<f64>,
    var_index: usize
) -> Vec<(&'static str, String)>
```

**Generated output:**
```tsx
<path fill="var(--fill-0, #440099)" />
<path stroke="var(--stroke-0, white)" />
```

**Benefits:**
- Easy theming without code changes
- Dark mode support
- Brand customization
- Fallback colors for browsers without CSS variable support

### 3. Figma SVG Export API Integration ✅

**File:** `src/tsx/figma_svg_export.rs`

Completed the implementation to fetch SVG content from Figma's export API:

```rust
async fn fetch_figma_svg(&self, file_key: &str, node_id: &str) 
    -> Result<String, SvgExportError>
```

**Process:**
1. Calls Figma's `/v1/images/{file_key}` endpoint
2. Gets temporary SVG URL
3. Downloads actual SVG content
4. Optionally writes to external file

**Benefits:**
- Accurate vector rendering
- Preserves complex effects
- No geometry calculation needed

### 4. Import Generation ✅

**File:** `src/tsx/generator.rs`

Automatically adds imports for external SVG path files:

```tsx
import React from 'react';
import svgPaths from "./svg-paths";

export function Component() {
  // ...
}
```

### 5. Configuration System ✅

**File:** `src/codegen_ext.rs`

Extended `CodeGenConfig` with new options:

```rust
pub struct CodeGenConfig {
    pub output_dir: PathBuf,
    pub separate_files: bool,
    pub exportable_only: bool,
    pub file_naming: FileNamingStrategy,
    
    // NEW OPTIONS:
    pub use_css_variables: bool,      // Enable CSS variable theming
    pub extract_svg_paths: bool,       // Extract complex paths
    pub svg_paths_filename: String,    // Name of paths file
}
```

**Usage example:**
```rust
let config = CodeGenConfig {
    output_dir: PathBuf::from("output"),
    separate_files: true,
    exportable_only: true,
    file_naming: FileNamingStrategy::ComponentName,
    use_css_variables: true,        // Enable theming
    extract_svg_paths: true,         // Extract paths
    svg_paths_filename: "svg-paths".to_string(),
};

let generator = TsxGenerator::new()
    .with_config(config.clone());
```

## Implementation Details

### Path Hashing Algorithm

Uses SHA256 to generate unique 8-character identifiers:

```rust
fn generate_path_hash(&self, path_data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(path_data.as_bytes());
    let result = hasher.finalize();
    format!("p{:x}", u32::from_be_bytes([result[0], result[1], result[2], result[3]]))
}
```

### Path Extraction Criteria

Paths are extracted if they meet complexity thresholds:

```rust
pub fn should_extract_path(&self, path_data: &str) -> bool {
    let command_count = path_data.matches(char::is_alphabetic).count();
    let length = path_data.len();
    
    command_count > 5 || length > 100
}
```

### Interior Mutability Pattern

Uses `RefCell` to allow mutation within immutable context:

```rust
pub struct TsxGenerator {
    path_registry: RefCell<PathRegistry>,
    // ...
}
```

This allows the generator to register paths during code generation without requiring `&mut self`.

## Comparison with Figma Make

| Feature | Figma Make | etch (Now) | Status |
|---------|------------|------------|--------|
| External path files | ✅ | ✅ | Implemented |
| CSS variables | ✅ | ✅ | Implemented |
| Path hashing | ✅ | ✅ | Implemented |
| Import generation | ✅ | ✅ | Implemented |
| Configuration | ✅ | ✅ | Implemented |
| SVG API integration | ✅ | ✅ | Implemented |

## Example Output

### Before:
```tsx
function Group() {
  return (
    <div className="...">
      <svg>
        <path d="M973.077 1169 C973.077 1195.23 968.5 1207.35 959.346 1207.35 C950.192..." fill="#440099" />
      </svg>
    </div>
  );
}
```

### After:
```tsx
import svgPaths from "./svg-paths";

function Group() {
  return (
    <div className="...">
      <svg>
        <path d={svgPaths.p2cf2ca00} fill="var(--fill-0, #440099)" />
      </svg>
    </div>
  );
}
```

## Future Enhancements

Potential additions to consider:

1. **Gradient Extraction**: Move complex gradients to CSS or separate definitions
2. **Filter Extraction**: Extract SVG filters to `<defs>` sections
3. **Animation Support**: Generate Framer Motion or CSS animations
4. **Responsive Breakpoints**: Generate responsive variants
5. **Component Variants**: Support Figma component variants
6. **Interactive States**: Handle hover/active/focus states

## Testing

Run tests with:
```bash
cargo test -p etch_figma
```

Test the path registry specifically:
```bash
cargo test -p etch_figma path_registry
```

## Usage in Production

To use these features in the Figma export binary:

```rust
let config = CodeGenConfig {
    output_dir: PathBuf::from("src/components"),
    separate_files: true,
    exportable_only: true,
    file_naming: FileNamingStrategy::ComponentName,
    use_css_variables: true,
    extract_svg_paths: true,
    svg_paths_filename: "svg-paths".to_string(),
};

let tsx_generator = TsxGenerator::new()
    .with_react_imports(true)
    .with_config(config.clone());

let codegen_system = CodeGenSystem::new(visitor, tsx_generator, config);
let result = codegen_system.generate_and_write()?;
```

## Dependencies Added

- `sha2 = "0.10"` - For path hashing

## Breaking Changes

None. All features are opt-in via configuration.

