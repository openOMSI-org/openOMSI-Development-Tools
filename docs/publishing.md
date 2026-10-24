# Publishing and signing

In the app:

1. **Signing keys**, then **New key**, once. Keep the secret key file private and make a backup
   copy. The upload button saves the public half (`.oopkey.pub`), which is safe to share.
2. **Build and publish**: turn on **Sign this build**, choose the key, press **Build**.
3. Share the single `.oop` file. Players drop it into openOMSI's `plugins` folder.
4. The game shows **signed by `<fingerprint>`**. If you publish your key's fingerprint somewhere
   players trust, they can confirm a plugin is really yours and has not been changed.

The **Inspector** opens any `.oop` (drop it on the window): its details, permissions and signature.
It can also sign a file that was built without a key.

From a terminal:

1. `oopc keygen` makes a key.
2. `oopc build --sign <your.oopkey>` builds signed.
3. `oopc verify <file.oop> --key <your.oopkey.pub>` checks a built file before you upload it.
