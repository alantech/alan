#!/usr/bin/env node

const { exec, } = require('child_process');
const path = require('path');

const pjson = require('./package.json');

const anycloudUrlBase = `https://github.com/alantech/alan/releases/download/v${pjson.version}/`;
let shell = '/bin/sh';
let request = '';
let extract = '';
let file = 'anycloud-ubuntu.tar.gz';
if (process.platform === 'darwin') {
  file = 'anycloud-macos.tar.gz';
  request = `curl -fOL ${anycloudUrlBase}${file}`;
  extract = `tar -xzf ${file}`;
} else if (process.platform === 'win32') {
  shell = 'powershell.exe';
  file = 'anycloud-windows.zip';
  request = `Invoke-WebRequest -OutFile anycloud-windows.zip -Uri ${anycloudUrlBase}${file}`;
  extract = 'Expand-Archive -Path anycloud-windows.zip -DestinationPath .';
} else {
  request = `curl -fOL ${anycloudUrlBase}${file}`;
  extract = `tar -xzf ${file}`;
}

exec('mkdir bin', (error, stdout, stderr) => {
  if (error) {
    console.log(stdout);
    console.error(stderr);
    process.exit(1);
  }
  const cwd = path.join(process.cwd(), 'bin')
  exec(request, { cwd, shell, }, (error, stdout, stderr) => {
    if (error) {
      console.log(stdout);
      console.error(stderr);
      process.exit(2);
    }
    exec(extract, { cwd, shell, }, (error, stdout, stderr) => {
      if (error) {
        console.log(stdout);
        console.error(stderr);
        process.exit(3);
      }
      if (shell === 'powershell.exe') {
        // Windows-specific mangling
        const fs = require('fs');
        fs.writeFileSync('./bin/anycloud', `#!/usr/bin/env node

const { exec, } = require('child_process');
const path = require('path');
const args = (() => {
  let argv = [ ...process.argv, ];
  argv.shift();
  argv.shift();
  return argv;
})();
exec(
  '& "' + path.join(__dirname, '\\anycloud.exe') + '" ' + args.join(' '),
  { shell: 'powershell.exe', },
  (error, stdout, stderr) => {
    if (stdout.length > 0) console.log(stdout);
    if (stderr.length > 0) console.error(stderr);
    process.exit(error);
  }
);
        `);
      }
    });
  });
});