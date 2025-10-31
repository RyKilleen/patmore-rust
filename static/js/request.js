const url = `/items`;

export const getItems = async () => {
  try {
    const resp = await fetch(url);
    const data = await resp.json();
    return data;
  } catch (err) {
    console.warn(err);
    return err;
  }
};

export const updateItem = async (label) => {
  try {
    const resp = await fetch(`${url}/${label}`, {
      method: "PATCH",
    })
      .then((response) => {
        if (!response.ok) {
          throw new Error(`HTTP error! Status: ${response.status}`);
        }
        return response.json(); // Rocket route returns plain text ("Item toggled" or "Item not found")
      })
      .catch((error) => {
        console.error("Error toggling item:", error);
      });

    // const data = await resp.json();
    return resp;
  } catch (err) {
    console.warn(err);
    return err;
  }
};
