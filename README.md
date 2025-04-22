# LAN Datensammler

Dieses Projekt beinhaltet eine Tauri-Applikation sowie ein Docker Stack zur Konfiguration von Prometheus mit Pushgateway und Grafana.  
Die Applikation sendet Informationen über Systemressourcen sowie laufende Anwendungen an das Pushgateway. Dadurch lässt sich über Grafana anzeigen, welches aktive System welche Anwendungen gestartet hat. Verwendet werden soll dieses Projekt auf LAN-Parties, um im Nachhinein semi-wertvolle Statistiken erheben zu können.

## lan-tracker

Eine Tauri-Applikation bestehend aus zwei Komponenten:

- **Metrics Exporter**: Erhebt Metriken und sendet diese an ein Pushgateway
- **Settings App**: Ermöglicht die Konfiguration von Benutzername, Pushgateway-URL und Autostart  

Sofern konfiguriert, startet die Anwendung bei Systemstart ohne Oberfläche, also nur mit der **Metrics Exporter**-Komponente. Es kann immer nur je eine Instanz der beiden Komponenten laufen.

## Grafana Stack

Ein Docker-Stack mit drei Komponenten:

- **Pushgateway** (Port: 9091): Empfängt Metriken der laufenden [lan-tracker](#lan-tracker)
- **Prometheus** (Port: 9090): Scraped Metriken vom **Pushgateway**
- **Grafana** (Port: 3000): Zeigt Dashboards zur Auswertung der erhobenen Daten an