# Snowhush dla GNOME

[English](README.md) · **Polski**

Śnieg na pulpicie GNOME. Pada spokojnie, osiada na górnych krawędziach okien i usypuje zaspy na dole ekranu. Kursor rozgarnia zaspy, przeciągane albo potrząsane okna zrzucają z siebie śnieg, a zamknięte okna zostawiają grudki, które rozsypują się w puch.

To port [Snowhusha](https://github.com/mbezruchko/Snowhush) autorstwa Mykhaila Bezruchki (macOS i Windows) na GNOME z Waylandem. Fizyka śniegu to jego kod, skopiowany bez zmian.

> Zrobione z pomocą AI. Sprawdzone na Ubuntu 26.04 z GNOME 50.

## Wymagania

- GNOME Shell 46–51 w sesji **Wayland**
- Karta graficzna z OpenGL ES 3 (każdy sterownik Intela, AMD albo NVIDII z ostatnich dziesięciu lat)

Inne pulpity z Waylandem mają własne porty: [KDE Plasma](https://github.com/TryCoffee/Snowhush/tree/kde-port) i [Hyprland](https://github.com/Nevers1/snow-hyprland).

## Instalacja

Wklej w terminalu to polecenie. Pobiera najnowsze wydanie, instaluje je i sprząta po sobie:

```bash
T=$(mktemp -d) && curl -fsSL -o "$T/s.zip" https://github.com/wojtipek/snowhush-gnome/releases/latest/download/snowhush-gnome.zip && unzip -q "$T/s.zip" -d "$T" && bash "$T/snowhush-gnome/install.sh"; rm -rf "$T"
```

Potem wyloguj się i zaloguj ponownie. Na Waylandzie GNOME wczytuje nowe rozszerzenia dopiero przy logowaniu.

To samo polecenie aktualizuje zainstalowaną wersję do najnowszej. Można też samemu pobrać `snowhush-gnome.zip` z [wydań (Releases)](https://github.com/wojtipek/snowhush-gnome/releases), rozpakować i uruchomić `install.sh`.

Wszystko trafia do jednego folderu: `~/.local/share/gnome-shell/extensions/snowhush-gnome@wojtipek`.

## Obsługa

Otwórz **Szybkie ustawienia** (prawy górny róg, tam gdzie Wi-Fi i Bluetooth) i znajdź kafelek **Śnieg**. Kliknięcie kafelka włącza i wyłącza śnieg, a strzałka obok otwiera menu:

- **Opady**: bez opadów, lekki, umiarkowany, gęsty albo zmienny
- **Zdmuchnij śnieg**: podmuch wiatru zwiewa cały śnieg, także z okien na innych obszarach roboczych
- **Opcje**: otwiera okno ustawień

W oknie ustawień (dostępnym też przy rozszerzeniu w aplikacji *Rozszerzenia*) są:

- **Gromadzenie**, **Topnienie**
- **Śnieg na oknach**, **Płatki nad oknami** (po wyłączeniu płatki chowają się za oknami, a śnieg na krawędziach okien zostaje)
- **Za oknami**: śnieg tylko na tapecie
- **Ostrzejszy obraz**: rysowanie w pełnej rozdzielczości ekranu zamiast w skali GNOME (więcej pracy dla karty graficznej)
- **Halloween**: od 30 października do świtu 1 listopada z zasp wstają duchy ze śniegu, w śniegu świecą dynie, a kursor niesie świeczkę (domyślnie włączone)
- **Język**: automatycznie (jak w systemie) albo jeden z 14 języków

Śnieg zatrzymuje się przy aplikacjach na pełnym ekranie, w Przeglądzie i na ekranie blokady. W trybie oszczędzania energii pada go mniej.

## Odinstalowanie

```bash
bash ~/.local/share/gnome-shell/extensions/snowhush-gnome@wojtipek/uninstall.sh
```

Polecenie usuwa rozszerzenie, silnik i ustawienia. Nic po nich nie zostaje.

## Języki

Menu jest w języku systemu: polskim (język podstawowy), angielskim (brytyjskim), hiszpańskim (meksykańskim), portugalskim (brazylijskim), katalońskim, niemieckim, czeskim, słowackim, ukraińskim, litewskim, węgierskim, chorwackim, esperanto albo łacinie. Każdy inny język dostaje łacinę. Język można też wybrać samemu: **Opcje** → **Język**.

Tłumaczeń nie pisali rodzimi użytkownicy tych języków. Poprawki i nowe języki są mile widziane. Wszystkie tłumaczenia są w jednej tabeli w pliku [`i18n.js`](extension/snowhush-gnome@wojtipek/i18n.js).

## Dla programistów

Opis działania i budowania ze źródeł jest w [angielskim README](README.md#how-it-works).

## Licencja

MIT, zobacz [LICENSE](LICENSE). Snowhush © Mykhailo Bezruchko.
