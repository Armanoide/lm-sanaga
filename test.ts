import { useQuery } from 'react-query'

const queryKey = ["test", 1, 2, { 3: true }] as const

const queryData = {
  innerData: "hola"
} as const

const { data, isLoading } = useQuery(
  ["example query"],
  () => Promise.resolve(queryData),
  {
    select: (data) => data.innerData,
    //         ^?
    onSuccess(data) {
      //          ^?
      console.log("Selected data:", data);
    }
  }
)
function getTime(): number {

  return new Date().getTime();
}
