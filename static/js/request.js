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
