# Migration von OSnap zu xsnap

Diese Anleitung beschreibt, wie du ein bestehendes OSnap-Projekt nach xsnap migrierst. Der `xsnap migrate` Command automatisiert den Großteil der Arbeit.

## Warum reicht Copy & Paste nicht?

OSnap und xsnap unterscheiden sich in drei Punkten, die eine einfache YAML-zu-JSON-Konvertierung nicht abdeckt:

### 1. Sizes: Strings vs. Objekte

OSnap erlaubt in Test-Dateien Size-Referenzen als **Strings**, die auf `defaultSizes` in der Config verweisen:

```yaml
# OSnap: String-Referenz
- name: Button--default
  url: /button
  sizes:
    - small
    - xlarge
```

xsnap erwartet **vollständige Size-Objekte** mit name, width und height:

```json
{
  "tests": [
    {
      "name": "Button--default",
      "url": "/button",
      "sizes": [
        { "name": "small", "width": 640, "height": 360 },
        { "name": "xlarge", "width": 1920, "height": 1080 }
      ]
    }
  ]
}
```

Der migrate Command löst die Strings automatisch gegen die `defaultSizes` aus `osnap.config.yaml` auf.

### 2. Snapshot-Dateinamen

OSnap und xsnap verwenden unterschiedliche Dateinamen für Baseline-Screenshots:

| Tool | Muster | Beispiel |
|------|--------|---------|
| OSnap | `{name}_{width}x{height}.png` | `Button--default_1920x1080.png` |
| xsnap | `{name}-{size}-{width}x{height}.png` | `Button--default-xlarge-1920x1080.png` |

xsnap fügt den **Size-Namen** in den Dateinamen ein. Ohne Umbenennung findet xsnap die bestehenden Baselines nicht.

### 3. Verzeichnisstruktur

| | OSnap | xsnap |
|---|-------|-------|
| Baselines | `__base_images__/` | `__base_images__/` |
| Aktuelle Screenshots | — | `__current__/` |
| Diffs | `__diff__/` | `__updated__/` |
| Aktualisierte Screenshots | `__updated__/` | `__updated__/` |

`__diff__/` und `__updated__/` sind generierte Artefakte und werden nicht migriert. Nur `__base_images__/` ist relevant.

## Voraussetzungen

- xsnap ist installiert (`xsnap --version`)
- Du befindest dich im Verzeichnis, das die `osnap.config.yaml` enthält

## Schritt-für-Schritt

### Ausgangslage

Typisches OSnap-Projekt:

```text
my-project/
  __image-snapshots__/              # snapshotDirectory (im Repo-Root)
    __base_images__/
      Button--default_375x211.png
      Button--default_640x360.png
      Button--default_1920x1080.png
    __diff__/                       # ignoriert
    __updated__/                    # ignoriert
  project/                          # App-Ordner
    osnap.config.yaml               # OSnap Config
    src/
      components/
        Button/
          Button-default.osnap.yaml # OSnap Test-Datei
```

### 1. Migration ausführen

```bash
cd my-project/project
xsnap migrate --source . --target .
```

`--source` und `--target` können identisch sein. Die neuen Dateien haben andere Dateiendungen (`.jsonc`, `.xsnap.json`) und überschreiben nichts.

### 2. Interaktive Bestätigung

Der Command fragt für jede Datei einzeln:

```text
Migrate ./osnap.config.yaml -> ./xsnap.config.jsonc? [Y/n]
Migrate ./src/components/Button/Button-default.osnap.yaml -> ./src/components/Button/Button-default.xsnap.json? [Y/n]
Rename Button--default_375x211.png -> Button--default-xsmall-375x211.png? [Y/n]
Rename Button--default_640x360.png -> Button--default-small-640x360.png? [Y/n]
Rename Button--default_1920x1080.png -> Button--default-xlarge-1920x1080.png? [Y/n]

Migration complete: 5 migrated, 0 skipped.
```

### 3. testPattern anpassen

Die migrierte Config übernimmt das OSnap testPattern (`src/**/*.osnap.yaml`). Das muss auf die neuen Dateien zeigen:

```jsonc
// xsnap.config.jsonc — manuell anpassen:
{
  "testPattern": "src/**/*.xsnap.json"
  // ...
}
```

### 4. Ergebnis prüfen

Nach der Migration sieht die Projektstruktur so aus:

```text
my-project/
  __image-snapshots__/
    __base_images__/
      Button--default-xsmall-375x211.png    # umbenannt
      Button--default-small-640x360.png     # umbenannt
      Button--default-xlarge-1920x1080.png  # umbenannt
  project/
    osnap.config.yaml                       # alt, kann gelöscht werden
    xsnap.config.jsonc                      # neu
    src/
      components/
        Button/
          Button-default.osnap.yaml         # alt, kann gelöscht werden
          Button-default.xsnap.json         # neu
```

Prüfe stichprobenartig:
- `xsnap.config.jsonc` enthält die richtigen `defaultSizes`, `snapshotDirectory`, etc.
- Die `.xsnap.json` Test-Dateien haben aufgelöste Sizes (Objekte mit name/width/height)
- Die Snapshot-Dateien in `__base_images__/` sind korrekt umbenannt

### 5. OSnap-Dateien aufräumen

Wenn alles stimmt:

```bash
# Alte Config löschen
rm osnap.config.yaml

# Alte Test-Dateien löschen
find . -name "*.osnap.yaml" -delete
find . -name "*.osnap.yml" -delete

# OSnap Diff-Artefakte löschen (optional)
rm -rf ../__image-snapshots__/__diff__
```

### 6. Testen

```bash
# Dev-Server starten (in einem separaten Terminal)
npm run dev

# xsnap Tests laufen lassen
xsnap test
```

Beim ersten Lauf sollten alle Tests **passen**, weil die umbenannten Baselines korrekt gefunden werden.

## Edge Cases

| Situation | Verhalten |
|-----------|-----------|
| Size-String nicht in `defaultSizes` | Warnung, String bleibt unaufgelöst |
| Snapshot-Dimensions passen zu keiner Size | Warnung, Datei wird nicht umbenannt |
| Zieldatei existiert bereits | Wird übersprungen |
| Kein `snapshotDirectory` in Config | Snapshot-Umbenennung wird übersprungen |
| Relativer `snapshotDirectory`-Pfad | Wird relativ zum `--source`-Verzeichnis aufgelöst |
| Kein `osnap.config.yaml` vorhanden | Nur Test-Dateien werden konvertiert (ohne Size-Auflösung) |

## Unterschiede OSnap vs. xsnap im Überblick

| | OSnap | xsnap |
|---|-------|-------|
| Config | `osnap.config.yaml` (YAML) | `xsnap.config.jsonc` (JSONC) |
| Tests | `*.osnap.yaml` (YAML) | `*.xsnap.json` (JSON) |
| Sizes in Tests | String-Referenz oder Objekt | Nur Objekt |
| Snapshot-Namen | `{name}_{WxH}.png` | `{name}-{size}-{WxH}.png` |
| Schema-Support | Nein | `$schema` mit Editor-Autocompletion |
