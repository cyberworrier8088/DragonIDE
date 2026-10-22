# DAY-29

Today i added a music player to DragonIDE!! 🎵

First i was thinking to add Spotify, but i dont have Spotify Premium 😭 and getting the Spotify integration working was not going how i wanted. Then i found Radio Browser and decided to make my own music panel using internet radio.

## First feature (Music Panel)

I added a new Music button in the sidebar. Now i can choose Lofi, Chill, Jazz, Synthwave, Classical and Ambient and play radio stations while coding.

The cool part is that it uses Radio Browser, a free radio station directory. No API key needed! I fetch the station list and use the HTML `<audio>` element to play the music.

I also added Play/Pause, Next and volume controls. The app saves my last vibe and volume using `localStorage`, so they stay saved after closing DragonIDE.

## Things i learned today

This feature took more learning than i expected hahaha.

- **Arrays `[]` and objects `{}`:** I used `{}` instead of a list in one place and the whole JS file stopped working. Took me a bit to understand what i did wrong.
- **`classList` and `className`:** I mixed these up too. `classList.add()` adds a class, but `className` is just a string. Small thing but it broke my code.
- **`localStorage`:** learned how to save the music vibe and volume, so users dont need to set everything again.
- **`textContent` and `innerHTML`:** Station names come from the internet. Using `textContent` helps display them as text instead of treating them as HTML.
- **Audio:** I learned how to change the stream URL, play/pause, control volume and handle stations that fail to play.

## How ChatGPT helped me

I used ChatGPT today to understand some confusing JS errors and learn things more easily. Sometimes i read the docs but still didn't understand why something worked that way. ChatGPT explained it in simpler words, and then i could go back to my code and fix the problem.

I still had to test things and figure out how they worked in my project. But it made the learning part easier instead of getting stuck on one small mistake for ages.

## Docs i used

- [Radio Browser API](https://docs.radio-browser.info/)
- [JavaScript Arrays](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Array)
- [JavaScript Objects](https://developer.mozilla.org/en-US/docs/Learn_web_development/Core/Scripting/Object_basics)
- [`classList`](https://developer.mozilla.org/en-US/docs/Web/API/Element/classList)
- [`localStorage`](https://developer.mozilla.org/en-US/docs/Web/API/Window/localStorage)
- [HTML Audio](https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Elements/audio)

## Why this took 1h

I spent around **1 hour building the music panel, connecting the API, fixing JS bugs and learning how the audio player works**.

The hardest part was not making the buttons. It was understanding why my code broke, reading docs, and fixing the mistakes without breaking other parts. ChatGPT helped me understand the confusing parts faster, so i could spend more time actually working on DragonIDE.

Now i can listen to music inside my own IDE while coding. Hahaha, nice little feature! 🦊🎵
