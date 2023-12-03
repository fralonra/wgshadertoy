# Translations

WgShadertoy uses [Fluent](https://projectfluent.org/) as the localization framework. If you are not familiar with it, we recommend that you read the Fluent documentation first.

## Edit Existing Translation

If you find a translation error (very likely), please help us improve it.

All translation files locate in the [`i18n`](https://github.com/fralonra/wgshadertoy/tree/master/i18n) folder. Please find the corresponding folder according to the [ISO 639 language code](https://en.wikipedia.org/wiki/List_of_ISO_639-1_codes), open the `wgshadertoy.ftl` file inside, modify the text, and then submit a PR. Thank you!

## Add A New Language

If you want to add a new language, please create a new folder in the `i18n` folder, named after the [ISO code of the language](https://en.wikipedia.org/wiki/List_of_ISO_639-1_codes), copy the [`wgshadertoy.ftl`](https://github.com/fralonra/wgshadertoy/blob/master/i18n/en/wgshadertoy.ftl) file in the `i18n/en` folder to this new folder, and modify it.

Next, you need to find the `LANGUAGES` constant in the [`src/i18n.rs`](https://github.com/fralonra/wgshadertoy/blob/master/src/i18n.rs) file and add the information of the new language properly.

If the new language uses a non-common writing system, the program may not be able to render the text correctly. You should open the [`src/fonts.rs`](https://github.com/fralonra/wgshadertoy/blob/master/src/fonts.rs) file and add the corresponding [writing system](https://en.wikipedia.org/wiki/Script_(Unicode)) and its common fonts in the `FONTS_MAP` constant.

After everything is done, submit a PR. 🍺!
