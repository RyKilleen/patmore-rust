 import { getItems } from './request.js';

const main = async () => {
  const items= await getItems();
  console.log(items)
}

// const renderItems = () => {
  
// }

main();
