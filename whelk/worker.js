let app;

onmessage = async function(e) {
    if (!app) {
        app = await
        import ('./pkg');
    }

    app.worker(e);
};