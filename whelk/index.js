async function entry() {
    if (Error.stackTraceLimit) {
        Error.stackTraceLimit = 50;
    }

    let app = await
    import ('./pkg');

    let worker = new Worker(new URL('./worker.js',
        import.meta.url));

    let terms = await (await (await fetch(new Request('/terms'))).blob()).arrayBuffer();

    let term = new Uint8Array(terms);
    app.entry(term, worker);
}

entry().catch(console.error)