// music panel.the stations come from radio-browser.info (free, no key)

const MUSIC_VIBES = [
    { name: "Lofi", tag: "lofi" },
    { name: "Chill", tag: "chillout" },
    { name: "Jazz", tag: "jazz" },
    { name: "Synthwave", tag: "synthwave" },
    { name: "Classical", tag: "classical" },
    { name: "Ambient", tag: "ambient" }
];

const RADIO_SERVERS = [
    "https://de1.api.radio-browser.info",
    "https://de2.api.radio-browser.info",
    "https://fi1.api.radio-browser.info"
];


let musicStations = [];
let currentStation = null;
let failedInARow = 0;

async function fetchStations(tag) {

    const query = `/json/stations/bytag/${encodeURIComponent(tag)}?hidebroken=true&order=clickcount&reverse=true&limit=60`;

    for (const server of RADIO_SERVERS) {
        try {
            const response = await fetch(server + query);
            if (!response.ok) continue;
            const stations = await response.json();

            return stations.filter(s => s.url_resolved?.startsWith("https://") && ["MP3", "AAC", "AAC+"].includes(s.codec));
        } catch (_) { }
    }

    throw new Error("Can't reach the radio server. Check your internet");
}

function renderVibes() {
    
    const box = document.getElementById("music-vibes");
    box.innerHTML = "";

    for (const vibe of MUSIC_VIBES) {
        const button = document.createElement("button");
        button.className = "music-vibe";
        button.dataset.tag = vibe.tag;
        button.textContent = vibe.name;
        button.addEventListener("click", () => loadVibe(vibe.tag));
        box.appendChild(button);
    }
}

async function loadVibe(tag) {
    document.querySelectorAll(".music-vibe").forEach(b => b.classList.toggle("active", b.dataset.tag === tag));

    const list = document.getElementById("music-stations");
    list.innerHTML = '<p class="music-hint">Loading...</p>';

    try {
        musicStations = await fetchStations(tag);
    } catch (error) {
        list.innerHTML = `<p class="music-hint">${error.message}</p>`;
        return;
    }

    localStorage.setItem("music-vibe", tag);
    renderStations();
}

function renderStations() {
    const list = document.getElementById("music-stations");
    list.innerHTML = "";

    if (musicStations.length === 0) {
        list.innerHTML = '<p class="music-hint">No stations for this vibe.</p>';
        return;
    }

    musicStations.forEach((station, index) => {

        const row = document.createElement("div");
        row.className = "music-station";
        if (station.stationuuid === currentStation?.stationuuid) row.classList.add("playing");

        // textContent, not innerHTML station name come from the internet
        row.textContent = station.name.trim();
        row.title = station.tags;
        row.addEventListener("click", () => playStation(index));
        list.appendChild(row);
    });
}

function playStation(index) {

    const station = musicStations[index];
    if (!station) return;

    currentStation = station;
    const audio = document.getElementById("music-audio");
    audio.src = station.url_resolved;
    audio.play().catch(() => { }); // error are handled

    document.getElementById("music-now").textContent = "Loading " + station.name.trim() + "...";
    renderStations();
}

function playNext() {
    if (musicStations.length === 0) return;
    const index = musicStations.findIndex(s => s.stationuuid === currentStation?.stationuuid);
    playStation((index + 1) % musicStations.length);
}

function togglePlay() {
    const audio = document.getElementById("music-audio");

    if (!currentStation) {
        playStation(0);
    } else if (audio.paused) {

        audio.src = currentStation.url_resolved;
        audio.play().catch(() => { });
    } else {
        audio.pause();
    }
}

function initMusic() {
    const audio = document.getElementById("music-audio");
    const volume = document.getElementById("music-volume");
    const playButton = document.getElementById("music-play-btn");

    document.getElementById("music-activity-btn")?.addEventListener("click", () => showSidebarView("music"));
    playButton.addEventListener("click", togglePlay);
    document.getElementById("music-next-btn").addEventListener("click", playNext);

    volume.value = localStorage.getItem("music-volume") ?? 50;
    audio.volume = volume.value / 100;
    volume.addEventListener("input", () => {
        audio.volume = volume.value / 100;
        localStorage.setItem("music-volume", volume.value);
    });

    audio.addEventListener("playing", () => {
        failedInARow = 0;
        playButton.textContent = "Pause";
        document.getElementById("music-now").textContent = "♪ " + currentStation.name.trim();
    });
    audio.addEventListener("pause", () => playButton.textContent = "Play");

    // a broken station skips to the next one, but stops after 5 so it cant loop forever
    audio.addEventListener("error", () => {
        if (!currentStation) return;
        failedInARow++;
        if (failedInARow < 5) {
            playNext();
        } else {
            failedInARow = 0;
            document.getElementById("music-now").textContent = "These stations are't working. Try anthor vibe.";
        }
    });

    renderVibes();
    loadVibe(localStorage.getItem("music-vibe") || "lofi"); // loads the list, doesn't play by itself
}

document.addEventListener("DOMContentLoaded", initMusic);