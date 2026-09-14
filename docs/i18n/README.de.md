<p align="center">
  <img src="../../desktop/src/assets/app-icon.png" width="88" alt="CueTuck">
</p>

# CueTuck · 唤词

**Your prompts, a shortcut away.**

[简体中文](../../README.md) · [English](README.en.md) · [日本語](README.ja.md) · [한국어](README.ko.md) · [Español](README.es.md) · [Français](README.fr.md) · **Deutsch**

Ein Desktop-Arbeitsplatz für Prompts mit lokaler Speicherung im Mittelpunkt. Sammle Vorlagen, Referenzbilder und Skills für Agenten, suche nach einem Prompt, fülle Variablen aus und kopiere das Ergebnis in dein KI-Werkzeug.

[Vorschauversion herunterladen](https://github.com/sutao2/CueTuck/releases) · [Dokumentation](../../docs/INDEX.md) · [Fehler melden](https://github.com/sutao2/CueTuck/issues)

![Prompt-Katalog mit Kategorien, Bildern, Suche und Filtern](../../docs/assets/readme/square.png)

## Funktionen

| Funktion | Möglichkeiten |
|---|---|
| Prompt-Katalog | Nach Kategorie, Modell oder Stichwort suchen. Bilder, Quellen und Autoren ansehen, Prompts herunterladen oder als Favoriten speichern. |
| Lokale Bibliothek | Inhalte kategorisieren, durchsuchen, favorisieren und Texte sowie Referenzen bearbeiten. Zwischen Karten und Listen wechseln. |
| Eigenständiger Launcher | Per einstellbarem globalem Tastenkürzel öffnen, suchen, Variablen ausfüllen und kopieren. Prompts erstellen, mit KI verbessern oder gezielt im Katalog suchen. |
| Übersetzung und KI | Im Katalog zwischen Chinesisch, Original und Englisch wechseln. Für lokale Übersetzung und Optimierung ein eigenes Modell aus der abgerufenen Modellliste auswählen. |
| Skills-Verwaltung | Öffentliche Quellen durchsuchen, lokale Skills finden, nach Agent und Geltungsbereich verwalten, installieren, Sicherungen wiederherstellen und manuell nach Updates suchen. |
| MCP-Anbindung | Kompatible Agenten können lokale Prompts suchen, lesen und mit Variablen ausfüllen. Werkzeuge für den öffentlichen Katalog sind optional. |

## Vom Ordnen zum Anwenden

### Vorlagen lokal aufbewahren

Die Desktop-Bibliothek verwendet SQLite. Vorlagen unterstützen `{{Variablen}}` und Referenzen. Melde dich an, wenn du Katalog-Favoriten, Veröffentlichungen oder die Synchronisierung deiner Bibliothek nutzen möchtest.

![Lokale Bibliothek mit Beispiel-Prompts und Modellangaben](../../docs/assets/readme/library.png)

### Ausfüllen, prüfen, kopieren

Verwende eine Vorlage mit einem neuen Ziel, einer anderen Zielgruppe oder neuem Material. Prüfe das fertige Ergebnis vor dem Kopieren. Die Vorlage bleibt wiederverwendbar.

![Vorschau eines ausgefüllten Prompts](../../docs/assets/readme/variables.png)

### Jederzeit per Tastenkürzel

Der Launcher hat ein eigenes Fenster und lässt sich per Tastatur bedienen. Fülle die Variablen aus und kopiere mit Enter. Aus neuen Eingaben kannst du auch Prompts erstellen oder eine KI-Optimierung starten.

![Variableneingabe im eigenständigen Launcher](../../docs/assets/readme/launcher.png)

> Die Screenshots zeigen den aktuellen Quellcode in der Browser-Vorschau. Der Katalog enthält öffentliche Inhalte; Bibliothek und Launcher verwenden Beispieldaten. Native SQLite-Nutzung, System-Tastenkürzel und Dateiinstallationen benötigen die Desktop-App. Veröffentlichte Pakete können hinter dem aktuellen Quellcode zurückliegen.

## Installation und Updates

Lade das passende Paket unter [GitHub Releases](https://github.com/sutao2/CueTuck/releases) herunter.

| Plattform | Paket | Unterstützung |
|---|---|---|
| macOS · Apple Silicon | `.dmg` | arm64, auf einem echten Mac geprüft |
| Windows | `.exe` | x64, Installation, Start und Deinstallation in CI geprüft |
| Linux | — | Noch nicht geprüft |

Unter **Einstellungen → Updates** kannst du Versionen prüfen, den Downloadfortschritt sehen und die Installation bestätigen. Die App ist eine Vorschauversion. macOS-Pakete sind ad hoc signiert und nicht von Apple notarisiert; Hinweise zu Systemmeldungen stehen in der [Installationsanleitung](../../deploy/README.md).

## Erste Schritte

1. Erstelle einen lokalen Prompt oder lade eine Vorlage aus dem Katalog herunter.
2. Wähle **Verwenden**, fülle Variablen aus und kopiere das Ergebnis in dein KI-Werkzeug.
3. Passe Tastenkürzel, Sprache und Darstellung in den Einstellungen an.
4. Trage unter **KI und Modelle** die Dienst-URL und deinen API-Schlüssel ein, rufe die Modellliste ab und wähle ein Modell.
5. Melde dich für Synchronisierung, Katalog-Favoriten oder Veröffentlichungen an und lege einen öffentlichen Spitznamen fest.

Lokale KI-Konfiguration bedeutet keine Offline-Inferenz. Die Desktop-App speichert Schlüssel im Anmeldedatenspeicher des Betriebssystems. Bei Übersetzung und Optimierung wird der betreffende Text an den gewählten Anbieter gesendet. Prüfe Text und öffentliche Anhänge vor der Veröffentlichung.

## Skills und MCP

Verwalte Skill-Ordner für Codex, Claude Code, Cursor, Pi und OpenCode, global oder je Projekt. Installation und Ersetzen umfassen Bestätigungs- und Sicherungsschritte. Eine Skill-Installation installiert keine MCP-Abhängigkeiten und führt keine Skripte aus. Siehe [Skills-Spezifikation](../../docs/specs/skills/spec.md).

MCP ist ein separat zu kompilierender **Rust-stdio-Dienst**. Für seine Ausführung sind weder Node.js noch uv nötig. Standardmäßig sind nur lokale Lese-Werkzeuge aktiv. Erzeuge die Konfiguration unter **Einstellungen → Netzwerk und Proxy → MCP-Anbindung für Agenten** und folge der [MCP-Anleitung](../../docs/how-to/mcp-clients.md).

## Entwicklung

Benötigt wird Node.js 22+. Für die native App brauchst du außerdem Rust und die plattformspezifischen Tauri-Build-Abhängigkeiten.

```bash
git clone https://github.com/sutao2/CueTuck.git
cd CueTuck/desktop
npm ci
npm test
npm run dev
```

Dies startet die Browser-Vorschau. Beende sie, bevor du die native App mit `npm run tauri dev` startest.

[Lokale Entwicklung](../../docs/how-to/local-dev.md) · [Bereitstellung und Releases](../../deploy/README.md) · [Mitwirken](../../CONTRIBUTING.md) · [Testkriterien](../../docs/reference/test-gates.md)

Die technische Dokumentation ist überwiegend auf Chinesisch. Nenne bei Fehlermeldungen Betriebssystem, App-Version und Reproduktionsschritte und entferne persönliche Daten aus Screenshots. Keine Schlüssel oder Sitzungstoken veröffentlichen.
