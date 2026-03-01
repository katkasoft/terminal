var term = new Terminal();
term.open(document.getElementById('terminal'));
const { listen } = window.__TAURI__.event;
const { invoke } = window.__TAURI__.core;

document.addEventListener('DOMContentLoaded', () => {
    invoke('write_to_pty', { data: "clear \n" });
});

term.onData(data => {
    invoke('write_to_pty', { data });
});

await listen('write', (event) => {
    term.write(event.payload);
}); 