import { getItems, updateItem } from "./request.js";

let s;
const main = async () => {
  refreshList();
  initWebsockets();
};

const initWebsockets = () => {
  s = new WebSocket(
    (window.location.protocol === "https:" ? "wss://" : "ws://") +
      window.location.host +
      "/ws/echo?channel",
  );

  s.onmessage = function (e) {
    console.log(e);
  };
  console.log({ s });
};

const refreshList = async () => {
  const items = await getItems();

  const itemListElement = getOrCreateList(items);
};

const handleCheck = async (label) => {
  const newItems = await updateItem(label);
  getOrCreateList(newItems);
};

const getOrCreateList = (items) => {
  const listId = "item-list";

  let itemListElement;
  let itemContainerExists = !!document.getElementById(listId);
  if (itemContainerExists) {
    itemListElement = document.getElementById(listId);
  } else {
    itemListElement = document.createElement("ul");
    itemListElement.id = listId;
    itemListElement.classList.add("list");
    itemListElement.addEventListener("click", (e) => {
      if (e.target.type !== "checkbox") {
        return;
      }
      handleCheck(e.target.value);
    });
    document.querySelector("body").appendChild(itemListElement);
  }

  let htmlString = items.map((x) => createListItemString(x)).join("");

  itemListElement.innerHTML = htmlString;

  return itemListElement;
};

const createListItemString = (item) => {
  return `
      
      <li>
        <label>
          <input type="checkbox" ${item.needed && "checked"} value="${item.label}" />${item.label}
        </label>
    </li>
    `;
};

main();
