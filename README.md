# fallout 2d20 character manager - rust edition

rebuilding the partially-functional svelte webapp in rust

many sins have probably been committed here unwittingly

this project isn't even remotely close to functional, if you can't tell what's going on and how to build/run it, you shouldn't be here

## roadmap

* 0.1 - base functionality development
    * ~~0.1.0 - initial setup~~
        * ~~0.1.1 - themes, main menu, db setup (postgres -> sqlite)~~
        * ~~0.1.2 - new_character page (origin/trait select)~~
        * ~~0.1.3 - special stat assignment~~ 
        * ~~0.1.4 - skills assignment~~ 
        * ~~0.1.5 - perk selection~~
        * ~~0.1.6 - stat calculation and review~~
        * ~~0.1.7 - equipment selection~~ `<- YOU ARE HERE`
        * 0.1.8 - review and acceptance page
        * 0.1.9 - cleanup and testing
    * 0.2.0 - full character build pages/logic
    * 0.3.0 - view character sheet, character saving, loading, importing, exporting
    * 0.4.0 - interactive character sheet
    * 0.5.0 - levelup workflow
    * 0.6.0 - inventory management
    * 0.7.0 - ui cleanup - ready for testing
        * consider dependency upgrade (such as sdl2 to sdl3)
* 1.0 - base functionality
    * 1.0.0 - mvp
    * 1.1.0 - gm tools - party viewing
    * 1.2.0 - gm tools - party stat managment
    * 1.3.0 - character history (backups)
    * 1.4.0 - consumable use logic
    * 1.5.0 - addiction/disease/exhaustion logic
    * 1.6.0 - mod handling - weapon builder
    * 1.7.0 - mod handling - apparel builder
    * 1.8.0 - ui cleanup - ready for testing
        * consider dependency upgrade (see above)
* 2.0 - player experience update
    * 2.0.0 - characters can be fully managed by players
    * 2.1.0 - npc data and management
    * 2.2.0 - npc builder
    * 2.3.0 - ally management (assign npcs to characters)
    * 2.4.0 - gm console - encounter tracking
    * 2.5.0 - gm console - extended test tracking
    * 2.6.0 - gm console - client/server LAN interactions (***BIG MAYBE***)
    * 2.7.0 - ui cleanup - ready for testing
        * consider dependency upgrade (see above)
* 3.0 - gm experience update
    * 3.0.0 - campaigns can be fully managed by gms
    * 3.1.0 - add cheetsheet and hint functionality
    * 3.2.0 - integrate references
        * maybe look at how chummer 5 handles books
    * 3.3.0 - add loot tables
    * 3.4.0 - add dice roller/loot table roller
    * 3.5.0 - add crafting guide
    * 3.6.0 - gm tools - storefront creation
    * 3.7.0 - storefront player view (likely dependent on LAN situation)
    * 3.8.0 - ui cleanup - ready for testing
        * consider dependency upgrade (see above)
* 4.0 - gameplay integration update
    * 4.0.0 - better utility for gameplay
    * 4.1.0 - add encounter tables
    * 4.2.0 - add scavenging locations and management
    * 4.3.0 - add settlement management
    * 4.4.0 - add faction managment
    * 4.5.0 - ui cleanup - ready for testing
        * consider dependency upgrade (see above)
* 5.0 - expanded rules update 1
    * 5.0.0 - support for expanded rules
    * 5.1.0 - add vehicles
    * 5.2.0 - add vehicle management
    * 5.3.0 - add specific junk
    * 5.4.0 - enable specific junk requirements for crafting
    * 5.5.0 - ui cleanup - ready for testing
        * consider dependency upgrade (see above)
* 6.0 - expanded rules update 2
    * 6.0.0 - further support for expanded rules
