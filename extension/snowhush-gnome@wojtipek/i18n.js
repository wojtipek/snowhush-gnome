import GLib from 'gi://GLib';

const POLISH = [
    'Śnieg',
    'Snowhush dla GNOME',
    'Bez opadów',
    'Lekki',
    'Umiarkowany',
    'Gęsty',
    'Zmienny',
    'Zdmuchnij śnieg',
    'Gromadzenie',
    'Topnienie',
    'Śnieg na oknach',
    'Płatki nad oknami',
    'Za oknami',
    'Ostrzejszy obraz',
    'Język',
    'Automatycznie',
    'Opcje',
];

export const TRANSLATIONS = {
    en: ['Snow', 'Snowhush for GNOME', 'No snowfall', 'Light', 'Moderate', 'Heavy', 'Variable',
        'Blow the snow away', 'Accumulation', 'Melting', 'Snow on windows', 'Flakes over windows',
        'Behind windows', 'Sharper image', 'Language', 'Automatic', 'Options'],
    ru: ['Снег', 'Snowhush для GNOME', 'Отключить', 'Легкий', 'Средний', 'Мощный', 'Разнообразный',
        'Сдуть снег', 'Сугробы', 'Таяние', 'Снег на окнах', 'Снежинки поверх окон',
        'Позади окон', 'Повышенная чёткость', 'Язык', 'Автоматически', 'Настройки']
    es: ['Nieve', 'Snowhush para GNOME', 'Sin nevada', 'Ligera', 'Moderada', 'Intensa', 'Variable',
        'Soplar la nieve', 'Acumulación', 'Deshielo', 'Nieve en las ventanas', 'Copos sobre las ventanas',
        'Detrás de las ventanas', 'Imagen más nítida', 'Idioma', 'Automático', 'Opciones'],
    pt: ['Neve', 'Snowhush para GNOME', 'Sem neve', 'Leve', 'Moderada', 'Intensa', 'Variável',
        'Soprar a neve', 'Acúmulo', 'Derretimento', 'Neve nas janelas', 'Flocos sobre as janelas',
        'Atrás das janelas', 'Imagem mais nítida', 'Idioma', 'Automático', 'Opções'],
    ca: ['Neu', 'Snowhush per a GNOME', 'Sense nevada', 'Lleugera', 'Moderada', 'Intensa', 'Variable',
        'Bufa la neu', 'Acumulació', 'Desglaç', 'Neu a les finestres', 'Flocs sobre les finestres',
        'Darrere les finestres', 'Imatge més nítida', 'Idioma', 'Automàtic', 'Opcions'],
    de: ['Schnee', 'Snowhush für GNOME', 'Kein Schneefall', 'Leicht', 'Mäßig', 'Stark', 'Wechselnd',
        'Schnee wegpusten', 'Ansammlung', 'Schmelzen', 'Schnee auf Fenstern', 'Flocken über Fenstern',
        'Hinter Fenstern', 'Schärferes Bild', 'Sprache', 'Automatisch', 'Optionen'],
    cs: ['Sníh', 'Snowhush pro GNOME', 'Bez sněžení', 'Slabé', 'Mírné', 'Husté', 'Proměnlivé',
        'Odfouknout sníh', 'Hromadění', 'Tání', 'Sníh na oknech', 'Vločky nad okny',
        'Za okny', 'Ostřejší obraz', 'Jazyk', 'Automaticky', 'Možnosti'],
    sk: ['Sneh', 'Snowhush pre GNOME', 'Bez sneženia', 'Slabé', 'Mierne', 'Husté', 'Premenlivé',
        'Odfúknuť sneh', 'Hromadenie', 'Topenie', 'Sneh na oknách', 'Vločky nad oknami',
        'Za oknami', 'Ostrejší obraz', 'Jazyk', 'Automaticky', 'Možnosti'],
    uk: ['Сніг', 'Snowhush для GNOME', 'Без снігопаду', 'Слабкий', 'Помірний', 'Сильний', 'Мінливий',
        'Здути сніг', 'Накопичення', 'Танення', 'Сніг на вікнах', 'Сніжинки над вікнами',
        'За вікнами', 'Чіткіше зображення', 'Мова', 'Автоматично', 'Параметри'],
    lt: ['Sniegas', 'Snowhush GNOME aplinkai', 'Be snygio', 'Silpnas', 'Vidutinis', 'Smarkus', 'Kintamas',
        'Nupūsti sniegą', 'Kaupimasis', 'Tirpimas', 'Sniegas ant langų', 'Snaigės virš langų',
        'Už langų', 'Ryškesnis vaizdas', 'Kalba', 'Automatiškai', 'Parinktys'],
    hu: ['Hó', 'Snowhush GNOME-hoz', 'Nincs havazás', 'Gyenge', 'Mérsékelt', 'Erős', 'Változó',
        'Hó elfújása', 'Felhalmozódás', 'Olvadás', 'Hó az ablakokon', 'Pelyhek az ablakok felett',
        'Ablakok mögött', 'Élesebb kép', 'Nyelv', 'Automatikus', 'Beállítások'],
    hr: ['Snijeg', 'Snowhush za GNOME', 'Bez snijega', 'Slab', 'Umjeren', 'Jak', 'Promjenjiv',
        'Otpuhni snijeg', 'Gomilanje', 'Otapanje', 'Snijeg na prozorima', 'Pahulje iznad prozora',
        'Iza prozora', 'Oštrija slika', 'Jezik', 'Automatski', 'Opcije'],
    eo: ['Neĝo', 'Snowhush por GNOME', 'Sen neĝado', 'Malforta', 'Modera', 'Forta', 'Varia',
        'Forblovi la neĝon', 'Amasiĝo', 'Degelo', 'Neĝo sur fenestroj', 'Flokoj super fenestroj',
        'Malantaŭ fenestroj', 'Pli akra bildo', 'Lingvo', 'Aŭtomate', 'Agordoj'],
    la: ['Nix', 'Snowhush ad GNOME', 'Sine nive', 'Levis', 'Modica', 'Densa', 'Varia',
        'Nivem diffla', 'Cumulatio', 'Liquefactio', 'Nix in fenestris', 'Flocci super fenestras',
        'Post fenestras', 'Imago acrior', 'Lingua', 'Sponte', 'Optiones'],
};

const FALLBACK = 'la';

export const LANGUAGES = [
    ['pl', 'Polski'],
    ['en', 'English (UK)'],
    ['ru', 'Русский'],
    ['es', 'Español (México)'],
    ['pt', 'Português (Brasil)'],
    ['ca', 'Català'],
    ['de', 'Deutsch'],
    ['cs', 'Čeština'],
    ['sk', 'Slovenčina'],
    ['uk', 'Українська'],
    ['lt', 'Lietuvių'],
    ['hu', 'Magyar'],
    ['hr', 'Hrvatski'],
    ['eo', 'Esperanto'],
    ['la', 'Latina'],
];

export function pickLanguage(names) {
    for (const name of names) {
        const base = name.split(/[._@]/)[0].toLowerCase();
        if (!base || base === 'c' || base === 'posix')
            continue;
        if (base === 'pl')
            return null;
        if (base in TRANSLATIONS)
            return base;
    }
    return FALLBACK;
}

let table = new Map();

export function setLanguage(code) {
    let language;
    if (code === 'pl')
        language = null;
    else if (code in TRANSLATIONS)
        language = code;
    else
        language = pickLanguage(GLib.get_language_names());
    table = new Map();
    if (language)
        POLISH.forEach((text, i) => table.set(text, TRANSLATIONS[language][i]));
}

setLanguage('');

export function _(text) {
    return table.get(text) ?? text;
}

export {POLISH};
