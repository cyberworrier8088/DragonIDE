# DAY-30

Today i added AI autocomplete and some other features to DragonIDE.

## 1. AI Autocomplete

![AI autocomplete](img/day23v1.png)

Finally added ghost text autocomplete!! When i type code and stop for a little time, it shows a grey suggestion. Press `Tab` to accept it and `Esc` to close it.

For now, i'm using my own Code AI API at `imu.hackclub.app`, with Qwen2.5-Coder. This is only temporary because i want to use my own FoxAI model in DragonIDE.

I was working on FoxAI, but it still needs more training. If i get the Pixl GPU/hosting grant, i want to train it more and host it so DragonIDE can use my own model instead. Hope i get that grant hahaha :')

This autocomplete was harder than i expected. I had to fix problems with typing, keyboard shortcuts and requests getting mixed up when i typed quickly. I also misspelled a variable name once and got a `ReferenceError`. One small typo and nothing worked 😭

## 2. Right-click menu

I removed the normal browser right-click menu from DragonIDE. Now it doesn't show Inspect, Reload and Back every time i right-click. I kept `Shift + right-click` for development.

## 3. Explorer improvements

I also added more options to the file explorer.

- Create files and folders
- Cut, copy and paste
- Rename and delete
- Copy full path or relative path

I fixed the paste behaviour too, so it doesn't overwrite an existing file. Copying folders also copies the files inside them.

I had to work on both Rust and JavaScript for this, and make sure the filesystem operations worked correctly.

I wanted to take screenshots of the explorer menu, but every time i clicked my screenshot button the menu closed. So no screenshot for this part today hahaha.

## Things i learned

I learned more about keyboard events, API requests, cancelling old requests, and connecting Rust commands with JavaScript. I also got a better idea of how to handle files and folders without accidentally overwriting things.

Some bugs took longer to find than actually writing the feature. I had to test things, find what was wrong, fix it, and test again.

FoxAI is still my goal for autocomplete. For now, Qwen is helping me get the feature working while i continue learning and improving my own model.

More updates tomorrow! 🦊
