import fileTree from '../src/file-tree';
import fs from 'fs';
import path from 'path';

// Convert the file tree to JSON
const fileTreeJson = JSON.stringify(fileTree, null, 2);

// Write to file-tree.json
fs.writeFileSync(path.join(__dirname, '../src/file-tree.json'), fileTreeJson);

console.log('File tree exported to file-tree.json successfully!');
