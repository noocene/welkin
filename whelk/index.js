async function entry() {
    let app = await
    import ('./pkg');

    let terms = await (await (await fetch(new Request('/terms'))).blob()).arrayBuffer();

    let term = new Uint8Array(terms);
    app.entry(term);
}

entry().catch(console.error)