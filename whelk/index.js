async function entry() {
    let app = await
    import ('./pkg');

    let term = new Uint8Array(await (await (await fetch(new Request('/term'))).blob()).arrayBuffer());
    app.entry(term);
}

entry().catch(console.error)