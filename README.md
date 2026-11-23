# Symulator fizyki 2D - własny silnik + macroquad

Natywna aplikacja w Rust z własnym (bez zewnętrznego silnika) silnikiem fizyki 2D.
Odpowiada za grawitację, opór powietrza, tarcie toczące, kolizje ze ścianami oraz
sprężyste zderzenia piłek z zachowaniem pędu i korekcją pozycji.

## Uruchomienie

Wymagany jest Rust (cargo); projekt zbudowano na wersji 1.100.

```bash
cargo build --release
./target/release/ball_physics_sim      # Linux / macOS
target\release\ball_physics_sim.exe    # Windows
```

Wersja developerska (szybszy kompilowaniem, wolniejsza w działaniu):

```bash
cargo run
```

## Sterowanie

| Akcja | Opis |
| --- | --- |
| **LPM** (klik w puste miejsce) | Nowa piłka o losowym promieniu i kolorze |
| **LPM** (przeciągnięcie) | Proca - linia celowania, podgląd trajektorii i wystrzał po puszczeniu |
| **PPM** (przytrzymanie i ruch) | Chwycenie piłki i rzut z pędem zgodnym z ruchem myszy |
| **E** | Eksplozja pod kursorem (animacja + impuls radialny) |
| **Przycisk Eksplozja** | Uzbrojenie celu - kolejny klik LPM w planszę wysadzi w wybranym miejscu |
| **Esc** / **PPM** | Anulowanie uzbrojonego celu |
| **Spacja** | Pauza / wznowienie |
| **R** | Reset planszy |

## Panel sterowania (lewy górny róg)

Suwaki:

- **Grawitacja** - 0 … 2000 px/s²
- **Restytucja** - 0 … 1 (sprężystość odbić)
- **Opór powietrza** - 0 … 2 1/s
- **Skala czasu** - 0.1x … 2.0x (slow motion / przyspieszenie)
- **Moc eksplozji** - 200 … 4000 (siła impulsu radialnego)

Przyciski: **Wyczysc**, **+20 pilek**, **Eksplozja** (uzbrojenie celu), **Wektory**
(wektory prędkości), **Pauza**, **Zapisz**, **Domyslne**.

Ustawienia zapisywane są w pliku `physics_settings.txt` w katalogu roboczym
(wczytywanie następuje automatycznie przy starcie). Przycisk **Domyslne**
przywraca wartości początkowe i również je zapisuje. Po każdej akcji w panelu
wyświetlany jest krótki komunikat (np. „Ustawienia zapisane").

W prawym górnym rogu wyświetlane są: FPS, liczba piłek i całkowita energia kinetyczna.

## Struktura kodu

- `src/main.rs` - punkt wejścia, akumulator czasowy, wejście, sterowanie eksplozją, render
- `src/ball.rs` - piłka, masa ∝ πr², ring buffer śladu, rysowanie
- `src/physics.rs` - silnik fizyki: całkowanie, kolizje, siatka przestrzenna, impulsy
- `src/effects.rs` - animacja eksplozji (pierścienie, błysk, promienie)
- `src/settings.rs` - zapis/odczyt ustawień i wartości domyślne
- `src/ui.rs` - własny HUD (suwaki, przyciski, statystyki)

Fizyka działa ze stałym krokiem 120 Hz (akumulator z limitem 16 podkroków na klatkę),
więc wynik nie zależy od aktualnego FPS. Animacja eksplozji odtwarzana jest w czasie
rzeczywistym, niezależnie od skali czasu symulacji.

Etykiety w HUDzie są zapisane bez polskich znaków diakrytycznych, ponieważ domyślny
font macroquad nie gwarantuje ich renderowania.
