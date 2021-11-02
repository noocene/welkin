async function entry() {
    if (Error.stackTraceLimit) {
        Error.stackTraceLimit = 50;
    }

    let app = await
    import ('./pkg');

    let terms = await (await (await fetch(new Request('/terms'))).blob()).arrayBuffer();

    let term = new Uint8Array(terms);
    app.entry(term);
}

entry().catch(console.error)