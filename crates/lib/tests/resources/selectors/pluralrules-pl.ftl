shared-photos =
    {$userName} {$userGender ->
        [male] dodał
        [female] dodała
       *[other] dodał(a)
    } {$photoCount ->
        [one] nowe zdjęcie
        [two] {$photoCount} nowe zdjęcia
        [few] {$photoCount} nowe zdjęcia
        [many] {$photoCount} nowych zdjęć
       *[other] {$photoCount} nowych zdjęć
    } do swojego strumienia.
