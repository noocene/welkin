let app;

onmessage = async function(e) {
    if (Error.stackTraceLimit) {
        Error.stackTraceLimit = 50;
    }

    if (!app) {
        app = await
        import ('./pkg');
    }

    app.worker(e);
};