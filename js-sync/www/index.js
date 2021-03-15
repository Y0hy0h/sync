import * as sync from "js-sync";

(async () => {
    const store = new sync.MemoryDb();

    console.log(await store.read("test"));
    await store.insert("test", "present");
    console.log(await store.read("test"));
})();

sync.fetch_flowers().then(blob => {
    document.getElementById("test_image").src = URL.createObjectURL(blob);
});