import { getItems, updateItem } from "./request.js";

const main = async () => {
  refreshList();
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
    itemListElement.addEventListener(
      "click",
      (e) => e.target.type == "checkbox" && handleCheck(e.target.value),
    );
    document.querySelector("body").appendChild(itemListElement);
  }

  let htmlString = items
    .map(
      (x) => `
      <li>
        <label>
          <input type="checkbox" ${x.needed && "checked"} value="${x.label}" />${x.label}
        </label>
    </li>`,
    )
    .join();

  itemListElement.innerHTML = htmlString;

  return itemListElement;
};

main();
