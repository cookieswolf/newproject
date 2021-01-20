import React, { useEffect, useState } from 'react';
import { Form, Grid } from 'semantic-ui-react';

import { useSubstrate } from './substrate-lib';
import { TxButton } from './substrate-lib/components';

import KittyCards from './KittyCards';

export default function Kitties (props) {
  const { api, keyring } = useSubstrate();
  const { accountPair } = props;

  const [kittyCnt, setKittyCnt] = useState(0);
  const [kittyDNAs, setKittyDNAs] = useState([]);
  const [kittyOwners, setKittyOwners] = useState([]);
  const [kittyPrices, setKittyPrices] = useState([]);
  const [kitties, setKitties] = useState([]);
  const [status, setStatus] = useState('');

  const fetchKittyCnt = () => {
    /* TODO: 加代码，从 substrate 端读取数据过来 */
    let unsubscribe;
    api.query.kittiesModule.kittiesCount(cnt => {
      console.log("kitties count: "+ cnt);
      setKittyCnt(cnt.toNumber());
    }).then(unsub => {
      unsubscribe = unsub;
    })
    .catch(console.error);

    return () => unsubscribe && unsubscribe();
  };

  const fetchKitties = () => {
    /* TODO: 加代码，从 substrate 端读取数据过来 */
    let unsubscribe;
    const kitty_ids = Array.from(Array(kittyCnt), (v, k) => k);

    api.query.kittiesModule.kitties.multi(kitty_ids, kitties => {
      //console.log("kitties1:", kitties);
      api.query.kittiesModule.kittyOwners.multi(kitty_ids, kitty_owners => {
        kitties.forEach(function (item, id, arr) {
          arr[id].id = id;
          arr[id].dna = item.unwrap();
          arr[id].owner = keyring.encodeAddress(kitty_owners[id].unwrap());
        })
        //console.log("kitties2:", kitties);
        setKittyOwners(kitty_owners);
        setKitties(kitties);
      }).then(unsub => {
        unsubscribe = unsub;
      }).catch(console.error);
    }).then(unsub => {
      unsubscribe = unsub;
    }).catch(console.error);

    return () => unsubscribe && unsubscribe();
  };

  const populateKitties = () => {
    /* TODO: 加代码，从 substrate 端读取数据过来 */
  };

  useEffect(fetchKittyCnt, [api, keyring]);
  useEffect(fetchKitties, [api, kittyCnt]);
  useEffect(populateKitties, [kittyDNAs, kittyOwners]);

  return <Grid.Column width={16}>
    <h1>小毛孩</h1>
    <KittyCards kitties={kitties} accountPair={accountPair} setStatus={setStatus}/>
    <Form style={{ margin: '1em 0' }}>
      <Form.Field style={{ textAlign: 'center' }}>
        <TxButton
          accountPair={accountPair} label='创建小毛孩' type='SIGNED-TX' setStatus={setStatus}
          attrs={{
            palletRpc: 'kittiesModule',
            callable: 'create',
            inputParams: [],
            paramFields: []
          }}
        />
      </Form.Field>
    </Form>
    <div style={{ overflowWrap: 'break-word' }}>{status}</div>
  </Grid.Column>;
}