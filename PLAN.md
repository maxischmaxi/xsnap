ich möchte ein snapshot tool entwickeln dass das osnap tool ersetzt. das tool ist auf https://github.com/eWert-Online/OSnap zu finden.
ich möchte xsnap in rust schreiben und ich möchte das die snapshot files in json und nicht im yaml format geschrieben werden. ich möchte auch das wir eine jsonc datei haben die mit der das schema klar definiert werden kann.
außerdem will ich das man pro snapshot oder auch global custom parameter bzw env variablen übergeben kann die an headless chrome/chromium weitergegeben werden können.
ich möchte auch wie bei osnap eine base config haben. außerdem möchte ich ein migration command bauen der alle alten snapshot yaml files nimmt und sie in json übersetzt.
außerdem möchte ich ein schönes terminal ui haben, ich will aber gleichzeitig auch das wir einen "pipeline" modus haben in dem das terminal ui nicht angezeigt wird, stattdessen für github ci pipelines der output optimiert wird.
