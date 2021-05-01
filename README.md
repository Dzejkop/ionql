# ionql

*WARNING* This is just a prototype and very WIP

What is an ion file? Refer to [ion-rs](https://github.com/pzol/ion_rs).

An SQL query processor for .ion files that you probably never needed or will need.

Currently only supports very basic projections of specific fields from one or more sections. Note that joins are not implemented yet, and sections must either be the same length, or one of the sections must be a dict section (like `CONTRACT`).

## Features overview:

1. [x] Basic projection, i.e. `SELECT a, b FROM section`
2. [x] Selecting from dict sections, i.e. `SELECT dates,purchase_id FROM CONTRACT, AVL.INV`
3. [ ] Filtering the results, i.e. `SELECT dates,room FROM AVL.INV WHERE room = 'DBL'`

    3.1 [ ] Equality operator

    3.2 [ ] Type aware operators (i.e. `>`, `<`, etc. for numerical values)

    3.3 [ ] Regex operator
4. [ ] Joins on sections (+ aliases), i.e. `SELECT i.dates, s.room FROM AVL.INV i JOIN AVL.STATE s ON (i.room = s.room)`
5. [ ] Support for `LIMIT`
6. [ ] Support for `ORDER BY`

# Basic usage
Let's consider a sample Ion
```
[DATA]
id = "data_id_01"
remark = "Some remark"
countries = ["Denmark", "Poland"]

[USERS]
|   name   |       mail       | age |
| -------- | ---------------- | --- |
| John Doe | johndoe@mail.com | 32  |
| Jane Doe | janedoe@mail.com | 44  |
```

We're interested in just the email and age of each user, but we'd also like to have on each row,
the remark that's in the `DATA` section.

Running the following query

```
> ionql ion.ion 'SELECT mail,age,remark FROM USERS,DATA'
```

Will return this results
```
| johndoe@mail.com | 32 | Some remark |
| janedoe@mail.com | 44 | Some remark |
```

Since this utility is Ion aware, each row can contain a valid Ion value. Therefore it's possible to also return the list of countries from the `DATA` section, like so

```
> ionql ion.ion 'SELECT countries,age,mail FROM USERS,DATA'
| [ "Denmark", "Poland" ] | 32 | johndoe@mail.com |
| [ "Denmark", "Poland" ] | 44 | janedoe@mail.com |
```

Note that we're free to reorder the columns from sections as we please.

# Section mapping

Ion files do not guarantee that sections will have a header. Sometimes headers might even contain invalid field names, therefore it's useful to be able to specify across all queries which sections have which fields in order.

The configuration file can be used to specify such section mappings, refer to the config file location dependign on the platform.

| platform |                                                     location |
| -------- | -----------------------------------------------------------: |
| linux    |                      `/home/$USER/.config/ionql/config.json` |
| osx      | `/Users/$USER/Library/Application Support/ionql/config.json` |
| windows  |           `C:\Users\$USER\AppData\Roaming\ionql\config.json` |

It's also possible to override the config file using the `--config` option.

An example config file looks like this:
```json
{
  "mappings": {
    "SECTION_NAME": {
      "dates": 0,
      "name": 1,
      "status": 2,
      "count": 3,
      "remark": 4
    }
  }
}
```
