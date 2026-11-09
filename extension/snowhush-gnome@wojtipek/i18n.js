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
    'Halloween',
    'Wolniej po 5 min bezczynności',
    'Pauza, gdy okno zasłania śnieg',
    'Wolniej na baterii',
    'Jesienne liście',
    'Liście',
    'Zdmuchnij liście',
    'Deszcz',
    'Od 30 października do 1 listopada',
    'Zawsze',
    'Wyłączone',
    'Kwiaty wiśni',
    'Płatki',
    'Zdmuchnij płatki',
    'Mieszane',
    'Intensywność',
    'Zdmuchnij wszystko',
    'Widoczność opadów',
    'Na dole ekranu',
    'Na oknach',
];

export const TRANSLATIONS = {
    en: ['Snow', 'Snowhush for GNOME', 'No snowfall', 'Light', 'Moderate', 'Heavy', 'Variable',
        'Blow the snow away', 'Accumulation', 'Melting', 'Snow on windows', 'Flakes over windows',
        'Behind windows', 'Sharper image', 'Language', 'Automatic', 'Options', 'Halloween', 'Slower after 5 min idle', 'Pause when a window hides the snow', 'Slower on battery', 'Autumn leaves', 'Leaves', 'Blow the leaves away', 'Rain', 'From 30 October to 1 November', 'Always', 'Off', 'Cherry blossom', 'Petals', 'Blow the petals away', 'Mixed', 'Intensity', 'Blow it all away', 'Visibility of what falls', 'At the bottom of the screen', 'On windows'],
    ru: ['Снег', 'Snowhush для GNOME', 'Отключить', 'Легкий', 'Средний', 'Мощный', 'Разнообразный',
        'Сдуть снег', 'Сугробы', 'Таяние', 'Снег на окнах', 'Снежинки поверх окон',
        'Позади окон', 'Повышенная чёткость', 'Язык', 'Автоматически', 'Настройки', 'Хеллоуин', 'Медленнее после 5 мин бездействия', 'Пауза, когда окно закрывает снег', 'Медленнее от батареи', 'Осенние листья', 'Листья', 'Сдуть листья', 'Дождь', 'С 30 октября по 1 ноября', 'Всегда', 'Выключено', 'Цветущая вишня', 'Лепестки', 'Сдуть лепестки', 'Смешанные', 'Интенсивность', 'Сдуть всё', 'Видимость осадков', 'Внизу экрана', 'На окнах'],
    es: ['Nieve', 'Snowhush para GNOME', 'Sin nevada', 'Ligera', 'Moderada', 'Intensa', 'Variable',
        'Soplar la nieve', 'Acumulación', 'Deshielo', 'Nieve en las ventanas', 'Copos sobre las ventanas',
        'Detrás de las ventanas', 'Imagen más nítida', 'Idioma', 'Automático', 'Opciones', 'Halloween', 'Más lento tras 5 min de inactividad', 'Pausa cuando una ventana tapa la nieve', 'Más lento con batería', 'Hojas de otoño', 'Hojas', 'Soplar las hojas', 'Lluvia', 'Del 30 de octubre al 1 de noviembre', 'Siempre', 'Desactivado', 'Flor de cerezo', 'Pétalos', 'Soplar los pétalos', 'Mezclado', 'Intensidad', 'Soplarlo todo', 'Visibilidad de lo que cae', 'Abajo de la pantalla', 'En las ventanas'],
    pt: ['Neve', 'Snowhush para GNOME', 'Sem neve', 'Leve', 'Moderada', 'Intensa', 'Variável',
        'Soprar a neve', 'Acúmulo', 'Derretimento', 'Neve nas janelas', 'Flocos sobre as janelas',
        'Atrás das janelas', 'Imagem mais nítida', 'Idioma', 'Automático', 'Opções', 'Halloween', 'Mais lento após 5 min de inatividade', 'Pausa quando uma janela cobre a neve', 'Mais lento na bateria', 'Folhas de outono', 'Folhas', 'Soprar as folhas', 'Chuva', 'De 30 de outubro a 1º de novembro', 'Sempre', 'Desativado', 'Flor de cerejeira', 'Pétalas', 'Soprar as pétalas', 'Misturado', 'Intensidade', 'Soprar tudo', 'Visibilidade do que cai', 'Na parte de baixo da tela', 'Nas janelas'],
    ca: ['Neu', 'Snowhush per a GNOME', 'Sense nevada', 'Lleugera', 'Moderada', 'Intensa', 'Variable',
        'Bufa la neu', 'Acumulació', 'Desglaç', 'Neu a les finestres', 'Flocs sobre les finestres',
        'Darrere les finestres', 'Imatge més nítida', 'Idioma', 'Automàtic', 'Opcions', 'Halloween', 'Més lent després de 5 min d’inactivitat', 'Pausa quan una finestra tapa la neu', 'Més lent amb bateria', 'Fulles de tardor', 'Fulles', 'Bufa les fulles', 'Pluja', 'Del 30 d’octubre a l’1 de novembre', 'Sempre', 'Desactivat', 'Flor de cirerer', 'Pètals', 'Bufa els pètals', 'Barrejat', 'Intensitat', 'Bufa-ho tot', 'Visibilitat del que cau', 'A la part de baix de la pantalla', 'A les finestres'],
    de: ['Schnee', 'Snowhush für GNOME', 'Kein Schneefall', 'Leicht', 'Mäßig', 'Stark', 'Wechselnd',
        'Schnee wegpusten', 'Ansammlung', 'Schmelzen', 'Schnee auf Fenstern', 'Flocken über Fenstern',
        'Hinter Fenstern', 'Schärferes Bild', 'Sprache', 'Automatisch', 'Optionen', 'Halloween', 'Langsamer nach 5 Min. Inaktivität', 'Pause, wenn ein Fenster den Schnee verdeckt', 'Langsamer im Akkubetrieb', 'Herbstlaub', 'Laub', 'Laub wegpusten', 'Regen', 'Vom 30. Oktober bis 1. November', 'Immer', 'Aus', 'Kirschblüte', 'Blütenblätter', 'Blütenblätter wegpusten', 'Gemischt', 'Stärke', 'Alles wegpusten', 'Sichtbarkeit des Niederschlags', 'Am unteren Bildschirmrand', 'Auf Fenstern'],
    cs: ['Sníh', 'Snowhush pro GNOME', 'Bez sněžení', 'Slabé', 'Mírné', 'Husté', 'Proměnlivé',
        'Odfouknout sníh', 'Hromadění', 'Tání', 'Sníh na oknech', 'Vločky nad okny',
        'Za okny', 'Ostřejší obraz', 'Jazyk', 'Automaticky', 'Možnosti', 'Halloween', 'Pomaleji po 5 min nečinnosti', 'Pozastavit, když okno zakryje sníh', 'Pomaleji na baterii', 'Podzimní listí', 'Listí', 'Odfouknout listí', 'Déšť', 'Od 30. října do 1. listopadu', 'Vždy', 'Vypnuto', 'Třešňové květy', 'Okvětní lístky', 'Odfouknout okvětní lístky', 'Smíšené', 'Intenzita', 'Odfouknout vše', 'Viditelnost srážek', 'Dole na obrazovce', 'Na oknech'],
    sk: ['Sneh', 'Snowhush pre GNOME', 'Bez sneženia', 'Slabé', 'Mierne', 'Husté', 'Premenlivé',
        'Odfúknuť sneh', 'Hromadenie', 'Topenie', 'Sneh na oknách', 'Vločky nad oknami',
        'Za oknami', 'Ostrejší obraz', 'Jazyk', 'Automaticky', 'Možnosti', 'Halloween', 'Pomalšie po 5 min nečinnosti', 'Pozastaviť, keď okno zakryje sneh', 'Pomalšie na batérii', 'Jesenné lístie', 'Lístie', 'Odfúknuť lístie', 'Dážď', 'Od 30. októbra do 1. novembra', 'Vždy', 'Vypnuté', 'Čerešňové kvety', 'Okvetné lístky', 'Odfúknuť okvetné lístky', 'Zmiešané', 'Intenzita', 'Odfúknuť všetko', 'Viditeľnosť zrážok', 'Dole na obrazovke', 'Na oknách'],
    uk: ['Сніг', 'Snowhush для GNOME', 'Без снігопаду', 'Слабкий', 'Помірний', 'Сильний', 'Мінливий',
        'Здути сніг', 'Накопичення', 'Танення', 'Сніг на вікнах', 'Сніжинки над вікнами',
        'За вікнами', 'Чіткіше зображення', 'Мова', 'Автоматично', 'Параметри', 'Гелловін', 'Повільніше після 5 хв бездіяльності', 'Пауза, коли вікно закриває сніг', 'Повільніше від батареї', 'Осіннє листя', 'Листя', 'Здути листя', 'Дощ', 'З 30 жовтня до 1 листопада', 'Завжди', 'Вимкнено', 'Цвіт вишні', 'Пелюстки', 'Здути пелюстки', 'Змішане', 'Інтенсивність', 'Здути все', 'Видимість опадів', 'Унизу екрана', 'На вікнах'],
    lt: ['Sniegas', 'Snowhush GNOME aplinkai', 'Be snygio', 'Silpnas', 'Vidutinis', 'Smarkus', 'Kintamas',
        'Nupūsti sniegą', 'Kaupimasis', 'Tirpimas', 'Sniegas ant langų', 'Snaigės virš langų',
        'Už langų', 'Ryškesnis vaizdas', 'Kalba', 'Automatiškai', 'Parinktys', 'Helovinas', 'Lėčiau po 5 min neveiklos', 'Pauzė, kai langas uždengia sniegą', 'Lėčiau naudojant bateriją', 'Rudens lapai', 'Lapai', 'Nupūsti lapus', 'Lietus', 'Nuo spalio 30 iki lapkričio 1 d.', 'Visada', 'Išjungta', 'Vyšnių žiedai', 'Žiedlapiai', 'Nupūsti žiedlapius', 'Mišrūs', 'Intensyvumas', 'Nupūsti viską', 'Kritulių matomumas', 'Ekrano apačioje', 'Ant langų'],
    hu: ['Hó', 'Snowhush GNOME-hoz', 'Nincs havazás', 'Gyenge', 'Mérsékelt', 'Erős', 'Változó',
        'Hó elfújása', 'Felhalmozódás', 'Olvadás', 'Hó az ablakokon', 'Pelyhek az ablakok felett',
        'Ablakok mögött', 'Élesebb kép', 'Nyelv', 'Automatikus', 'Beállítások', 'Halloween', 'Lassabb 5 perc tétlenség után', 'Szünet, ha ablak takarja a havat', 'Lassabb akkumulátorról', 'Őszi levelek', 'Levelek', 'Levelek elfújása', 'Eső', 'Október 30-tól november 1-ig', 'Mindig', 'Kikapcsolva', 'Cseresznyevirág', 'Szirmok', 'Szirmok elfújása', 'Vegyes', 'Intenzitás', 'Mindent elfúj', 'A hulló dolgok láthatósága', 'A képernyő alján', 'Az ablakokon'],
    hr: ['Snijeg', 'Snowhush za GNOME', 'Bez snijega', 'Slab', 'Umjeren', 'Jak', 'Promjenjiv',
        'Otpuhni snijeg', 'Gomilanje', 'Otapanje', 'Snijeg na prozorima', 'Pahulje iznad prozora',
        'Iza prozora', 'Oštrija slika', 'Jezik', 'Automatski', 'Opcije', 'Noć vještica', 'Sporije nakon 5 min neaktivnosti', 'Pauza kad prozor prekrije snijeg', 'Sporije na bateriji', 'Jesensko lišće', 'Lišće', 'Otpuhni lišće', 'Kiša', 'Od 30. listopada do 1. studenoga', 'Uvijek', 'Isključeno', 'Cvijet trešnje', 'Latice', 'Otpuhni latice', 'Miješano', 'Jačina', 'Otpuhni sve', 'Vidljivost oborina', 'Na dnu zaslona', 'Na prozorima'],
    eo: ['Neĝo', 'Snowhush por GNOME', 'Sen neĝado', 'Malforta', 'Modera', 'Forta', 'Varia',
        'Forblovi la neĝon', 'Amasiĝo', 'Degelo', 'Neĝo sur fenestroj', 'Flokoj super fenestroj',
        'Malantaŭ fenestroj', 'Pli akra bildo', 'Lingvo', 'Aŭtomate', 'Agordoj', 'Halovino', 'Pli malrapide post 5 min da neaktiveco', 'Paŭzo, kiam fenestro kovras la neĝon', 'Pli malrapide per baterio', 'Aŭtunaj folioj', 'Folioj', 'Forblovi la foliojn', 'Pluvo', 'De la 30-a de oktobro ĝis la 1-a de novembro', 'Ĉiam', 'Malŝaltita', 'Ĉerizaj floroj', 'Petaloj', 'Forblovi la petalojn', 'Miksite', 'Intenso', 'Forblovi ĉion', 'Videbleco de tio, kio falas', 'Ĉe la malsupro de la ekrano', 'Sur fenestroj'],
    la: ['Nix', 'Snowhush ad GNOME', 'Sine nive', 'Levis', 'Modica', 'Densa', 'Varia',
        'Nivem diffla', 'Cumulatio', 'Liquefactio', 'Nix in fenestris', 'Flocci super fenestras',
        'Post fenestras', 'Imago acrior', 'Lingua', 'Sponte', 'Optiones', 'Halloween', 'Tardius post 5 min otii', 'Pausa cum fenestra nivem tegit', 'Tardius cum pila electrica', 'Folia autumnalia', 'Folia', 'Folia diffla', 'Pluvia', 'A die XXX Octobris ad diem I Novembris', 'Semper', 'Exstinctum', 'Flores cerasi', 'Petala', 'Petala diffla', 'Mixta', 'Vis', 'Omnia diffla', 'Quae cadunt quam conspicua', 'In imo scrinio', 'In fenestris'],
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
