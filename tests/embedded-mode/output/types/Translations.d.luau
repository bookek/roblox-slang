--[[
    Roblox Slang - Type Definitions
    
    This file provides type definitions for Luau LSP autocomplete.
    DO NOT MODIFY BY HAND - Your changes will be overwritten!
    
    Generated from translation files in your project.
    To update type definitions, edit your JSON/YAML files and run:
        roblox-slang build
    
    Usage:
    Place this file alongside your Translations.lua module.
    Your IDE/LSP will automatically provide autocomplete and type checking.
    
    Learn more: https://github.com/mathtechstudio/roblox-slang
--]]

export type Translations = {
    new: (locale: string?) -> TranslationsInstance,
}

export type TranslationsInstance = {
    _locale: string,
    _translator: any,
    _localeChangedCallbacks: {any},

    setLocale: (self: TranslationsInstance, locale: string) -> (),
    getLocale: (self: TranslationsInstance) -> string,
    onLocaleChanged: (self: TranslationsInstance, callback: (newLocale: string, oldLocale: string) -> ()) -> (),
    getAsset: (self: TranslationsInstance, assetKey: string) -> string,

    ui_buttons_buy: (self: TranslationsInstance) -> string,
    ui_buttons_cancel: (self: TranslationsInstance) -> string,
    ui_buttons_sell: (self: TranslationsInstance) -> string,
    ui_labels_welcome: (self: TranslationsInstance) -> string,

    ui: {
        buttons: {
            buy: (self: TranslationsInstance) -> string,
            cancel: (self: TranslationsInstance) -> string,
            sell: (self: TranslationsInstance) -> string,
        },
        labels: {
            welcome: (self: TranslationsInstance) -> string,
        },
    },
}
