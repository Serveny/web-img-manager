const fs = require('fs');
const path = require('path');

const inputDir = './src/bindings/';
const outputFile = './src/rs-bindings.ts';

const readFileWithoutFirstLine = (filePath) => {
  return new Promise((resolve, reject) => {
    fs.readFile(filePath, 'utf8', (err, data) => {
      if (err) return reject(err);
      const lines = data.split('\n');
      lines.shift(); // Skip first line
      resolve(lines.join('\n'));
    });
  });
};

const processFiles = async () => {
  try {
    const files = fs.readdirSync(inputDir);
    let combinedData = '';

    for (const file of files) {
      const filePath = path.join(inputDir, file);
      const fileData = await readFileWithoutFirstLine(filePath);
      combinedData += fileData + '\n';
    }

    fs.writeFileSync(outputFile, combinedData, 'utf8');
    console.log('Files combined.');
  } catch (err) {
    console.error('Error:', err);
  }
};

processFiles();
