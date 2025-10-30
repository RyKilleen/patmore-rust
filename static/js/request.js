export const getItems =  async (lat, lng) => {
    const url = `/items`

    try {

        const resp = await fetch(url);
        const data = await resp.json();
        return data;
    } catch (err) {
        console.warn(err)
        return err;
    }   
} 
