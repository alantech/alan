## Install via NPM

```
npm install -g anycloud
```

## Install per OS

### MacOS via Homebrew

```bash
brew install alantech/homebrew-core/anycloud
```

For Linux and Windows it is recommended to install AnyCloud via the [published artifacts](https://github.com/alantech/alan/releases/latest). Simply download the zip or tar.gz file for your operating system, and extract the `anycloud` executable to somewhere in your `$PATH`, make sure it's marked executable \(if not on Windows\), and you're ready to roll.

### Linux

```bash
wget https://github.com/alantech/alan/releases/latest/download/anycloud-ubuntu.tar.gz
tar -xzf anycloud-ubuntu.tar.gz
sudo mv anycloud /usr/local/bin/anycloud
```

### Windows PowerShell

```bash
Invoke-WebRequest -OutFile anycloud-windows.zip -Uri https://github.com/alantech/alan/releases/latest/download/anycloud-windows.zip
Expand-Archive -Path anycloud-windows.zip -DestinationPath C:\windows
```
