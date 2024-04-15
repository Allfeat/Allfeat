// Import
import '@allfeat/types/src/interfaces/augment-api';
import '@allfeat/types/src/interfaces/augment-types';

import { ApiPromise, WsProvider } from '@polkadot/api';

const wsProvider = new WsProvider();

async function query_artist_data() {
    // Construct
    const api = await ApiPromise.create({ provider: wsProvider });

    const artist_data = await api.query.artists.artistOf("0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac");

    console.log(artist_data.toHuman())
}

query_artist_data().catch();