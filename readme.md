# tactile-win

`tactile-win` is a personal project written in Rust, created to explore the language and Windows APIs, as well as the use of LLMs as assistance in the development process since it's the first time I am developing an application using an agent-based approach.

The application is inspired by the GNOME extension 'Tactile', a grid-based window resizing solution, but it's implemented as a native Windows application because I miss the feature a lot when I'm using my Windows PC.

The goal of this project is to:
- Explore the Rust language and build ecosystem by building a non-trivial application of my interest
- Explore Windows-specific APIs (WinAPI)
- Experiment with window management and desktop automation concepts
- Explore about and experiment with modern AI assisted development workflows
- Fully build and release a working application
- LEARN

---

**This project is currently in an early development stage and is primarily intended as a learning exercise.**

---

## Try the application

1. **Clone the repository**
	
```
git clone https://github.com/diegolas-code/tactile-win.git
cd tactile-win
```
2. **Build the project**
	
```
cargo build --release
```
3. **Run the application**
	
```
cargo run --release
```
4. **Try it out!** While the application is running, try resizing any open window using the `Ctrl+Alt+F9` hotkey. Enter any two letters from the grid to select the desired area corners. Application remains open and available to use on any window until stopped in the terminal with `Ctrl+C`.

## License

Licensed under the [MIT License](LICENSE).