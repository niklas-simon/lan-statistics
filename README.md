# LAN Manager

Dieses Projekt beinhaltet eine Tauri-Applikation als Frontend (`src` und `src-tauri`) sowie einen Rust-Server als Backend (`src-server`).  
Die Tauri-App scannt den Host nach laufenden Spielen (Whitelist unter `src-server/games.json`). Diese werden an den Server gesendet. Der Server antwortet mit einer Liste aller Spiele, welche alle Clients zusammen spielen. Die Tauri-App stellt dies im Frontend dar.  
Die App ist gedacht für LAN-Parties, um jedem Spieler mitzuteilen, was die anderen spielen. Sollte ein Spieler über längere Zeit nicht das spielen, was die Mehrheit spielt, bekommt er eine Erinnerung (Windows-Notification).