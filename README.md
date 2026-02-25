
<div align="center">
    <br/>
    <p>
        <img src="/resources/icon.png"
            title="Rift" alt="Logo" width="120" />
        <h1>Rift</h1>
    </p>
    <p>
        Rift is a beautiful privacy‑first music player.<br/>
        Enjoy your music without distractions, ads, or tracking.
    <br/>
    <br/>
    <img src="/resources/screenshot-01.png"
        title="Interface Screenshot 1" alt="Screenshot 1" width="700" />
    <br/>
    <img src="/resources/screenshot-02.png"
        title="Interface Screenshot 2" alt="Screenshot 2" width="700" />
</div>

> [!WARNING]
> Project is still in a raw early stage. Expect rough edges, incomplete features, and breaking changes.

## Build from source
### Requirements
-   [Node.js](https://nodejs.org/)
-   [Rust](https://rust-lang.org/)
-   [Tauri](https://tauri.app/)

1. Clone repository:
```zsh
git clone https://github.com/wdkqqq/rift.git
cd Rift
```
2. Install dependencies:
```zsh
bun install
# or
pnpm install
# or
npm install
```
3. Run application

```zsh
bun tauri dev
# or
pnpm tauri dev
# or
npm run tauri dev
```
4. Build application
```zsh
bun tauri build
# or
pnpm tauri build
# or
npm run tauri build
```
## Contributing

We are always welcome to contributions! See [CONTRIBUTING.md](https://github.com/wdkqqq/rift/blob/main/CONTRIBUTING.md) for contributing guide.

## License

[GPL-3.0 license](https://github.com/wdkqqq/rift/blob/main/LICENSE)
