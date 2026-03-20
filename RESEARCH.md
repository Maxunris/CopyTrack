# Clipboard App Research

Этот файл собирает функции похожих clipboard-приложений, которые хорошо подходят CopyTrack и могут стать частью следующих итераций.

## Что уже подтверждено как полезное

### 1. Keyboard-first сценарий

Похожие продукты выигрывают тогда, когда история вызывается по хоткею, быстро фильтруется поиском и не заставляет тянуться к мышке. Для CopyTrack это значит, что quick access должен оставаться самым быстрым сценарием, а не второстепенным.

### 2. Коллекции поверх обычной истории

Paste развивает идею “pinboards”: не просто длинная лента копирований, а отдельные постоянные коллекции для шаблонов, команд, ссылок и reusable snippets. Для CopyTrack это хорошо ложится в будущие pinned collections.

### 3. Минимализм и скорость

Maccy делает ставку на lightweight, keyboard-first и native UI. Это хороший ориентир для нас: меньше визуального шума, меньше лишних действий, больше фокуса на recall flow.

### 4. Богатые превью и распознавание источника

Raycast развивает визуальные превью для ссылок, favicon/domain-подсказки, отдельную обработку файлов и корректное сохранение source app. Для CopyTrack сюда хорошо подходят richer previews для ссылок, better file cards и расширенная метаинформация.

### 5. Умное игнорирование чувствительных копирований

Maccy подчеркивает ignore flows для конфиденциальных типов и временное отключение захвата. Для CopyTrack это подтверждает ценность excluded apps, pause capture и будущего vault mode.

### 6. Импорт, экспорт и переносимость

Raycast и другие инструменты поддерживают импорт JSON-форматов для reusable content. Это хороший сигнал, что portability — не вторичная фича, а часть доверия к продукту.

## Что стоит рассмотреть для CopyTrack дальше

- Pinned collections или folders поверх обычной истории
- Rich link preview с favicon и доменом
- Paste without formatting как отдельное действие
- Ignore next copy / temporary pause without входа в настройки
- Отдельный режим для snippets и шаблонов
- Быстрые действия в menu bar меню для последних элементов
- Better file preview для нескольких файлов сразу
- Vault mode для чувствительных записей
- Optional cloud sync только после стабилизации local-first ядра

## Источники

- Maccy: https://github.com/p0deje/Maccy
- Paste Pinboards: https://pasteapp.io/help/organize-with-pinboards
- Raycast changelog and clipboard improvements: https://www.raycast.com/changelog/macos/7
