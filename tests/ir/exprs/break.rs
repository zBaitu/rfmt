fn f() {
    'label: for a in b {
        break 'label;
    }

    let a = for b in c {
        break 0;
    };
}
