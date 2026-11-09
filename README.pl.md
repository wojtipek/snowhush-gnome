# Snowhush dla GNOME

[English](README.md) · **Polski** · [Русский](README.ru.md)

Śnieg na pulpicie GNOME. Pada spokojnie, osiada na górnych krawędziach okien i usypuje zaspy na dole ekranu. Kursor rozgarnia zaspy, przeciągane albo potrząsane okna zrzucają z siebie śnieg, a zamknięte okna zostawiają grudki, które rozsypują się w puch.

To port [Snowhusha](https://github.com/mbezruchko/Snowhush) autorstwa Mykhaila Bezruchki (macOS i Windows) na GNOME z Waylandem. Fizyka śniegu to jego kod, skopiowany bez zmian.

> Zrobione z pomocą AI. Sprawdzone na Ubuntu 26.04 z GNOME 50.

## Wymagania

- GNOME Shell 46–51 w sesji **Wayland**
- Karta graficzna z OpenGL ES 3 (każdy sterownik Intela, AMD albo NVIDII z ostatnich dziesięciu lat)

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

- **Śnieg**, **Jesienne liście**, **Deszcz** albo **Kwiaty wiśni**: co pada. Przy zmianie podmuch zwiewa to, co leży, a potem z góry zaczyna padać nowe. Kafelek zmienia wtedy nazwę i ikonę. Deszcz rozpryskuje się na oknach i spływa kroplami po ich szybach
- **Mieszane**: dwie pogody naraz, na przykład śnieg z deszczem. Po włączeniu zaznacza się dwie, a zaznaczenie trzeciej odznacza tę wybraną najdawniej
- **Gromadzenie**: gdzie śnieg osiada: **Na dole ekranu** i **Na oknach** (każde z osobna można wyłączyć; deszcz i tak rozpryskuje się na dole)
- **Intensywność**: suwak od 0 do 100%
- **Zdmuchnij śnieg** (albo **Zdmuchnij liście** czy **Zdmuchnij płatki**; przy deszczu tej pozycji nie ma): podmuch wiatru zwiewa cały śnieg, także z okien na innych obszarach roboczych
- **Opcje**: otwiera okno ustawień

W oknie ustawień (dostępnym też przy rozszerzeniu w aplikacji *Rozszerzenia*) są:

- **Topnienie**
- **Płatki nad oknami** (po wyłączeniu płatki chowają się za oknami, a śnieg na krawędziach okien zostaje)
- **Za oknami**: śnieg tylko na tapecie
- **Ostrzejszy obraz**: rysowanie w pełnej rozdzielczości ekranu zamiast w skali GNOME (więcej pracy dla karty graficznej)
- **Widoczność opadów**: suwak od 10 do 100%, przy mniejszych wartościach to, co spada, jest bardziej przezroczyste
- **Halloween**: z zasp wstają duchy ze śniegu, w śniegu świecą dynie, a kursor niesie świeczkę. Do wyboru: od 30 października do 1 listopada (domyślnie), zawsze albo wyłączone. Działa tylko przy śniegu
- **Wolniej po 5 min bezczynności**: po pięciu minutach bez ruchu myszą i klawiaturą śnieg zwalnia do 10 klatek na sekundę i przyspiesza, gdy wrócisz
- **Pauza, gdy okno zasłania śnieg**: przy wyłączonych **Płatkach nad oknami** (albo włączonym **Za oknami**) śnieg staje, gdy okno zasłania cały ekran
- **Wolniej na baterii**: mniej klatek na sekundę bez zasilacza (15 przy spokojnym padaniu, najwyżej 30)
- **Język**: automatycznie (jak w systemie) albo jeden z 15 języków

Śnieg zatrzymuje się przy aplikacjach na pełnym ekranie, w Przeglądzie i na ekranie blokady. W trybie oszczędzania energii pada go mniej.

## Odinstalowanie

```bash
bash ~/.local/share/gnome-shell/extensions/snowhush-gnome@wojtipek/uninstall.sh
```

Polecenie usuwa rozszerzenie, silnik i ustawienia. Nic po nich nie zostaje.

## Języki

Menu jest w języku systemu: polskim (język podstawowy), angielskim (brytyjskim), rosyjskim, hiszpańskim (meksykańskim), portugalskim (brazylijskim), katalońskim, niemieckim, czeskim, słowackim, ukraińskim, litewskim, węgierskim, chorwackim, esperanto albo łacinie. Każdy inny język dostaje łacinę. Język można też wybrać samemu: **Opcje** → **Język**.

Większość tłumaczeń nie została napisana przez rodzimych użytkowników tych języków. Poprawki i nowe języki są mile widziane. Wszystkie tłumaczenia są w jednej tabeli w pliku [`i18n.js`](extension/snowhush-gnome@wojtipek/i18n.js).

## Dla programistów

Opis działania i budowania ze źródeł jest w [angielskim README](README.md#how-it-works).

## Licencja

Własny kod portu (rozszerzenie GNOME i silnik wokół fizyki) jest na licencji MIT, zobacz [LICENSE](LICENSE). Fizyka (`snow-core/src`), shadery i obrazki na Halloween pochodzą ze Snowhusha © Mykhailo Bezruchko i są na licencji [PolyForm Noncommercial 1.0.0](LICENSE-SNOWHUSH): można je za darmo używać, udostępniać i zmieniać, ale nie sprzedawać. Dlatego całej paczki też nie wolno sprzedawać.
