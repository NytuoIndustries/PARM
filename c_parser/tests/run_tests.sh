the_c_file=$1
findalltheS=$(find . -name "*.s")
chmod +x $the_c_file

for the_s_file in $findalltheS
do
    ./$the_c_file $the_s_file
done
 

allnormalbin=$(find . -name "*.bin")
goodcounter=0
for normalbin in $allnormalbin
do
if [[ $normalbin != *_origin.bin ]]
then
    originbin=${normalbin%.*}_origin.bin
    if cmp -s $normalbin $originbin
    then
        echo "Test $normalbin passed"
        goodcounter=$((goodcounter+1))
    else
        echo "Test $normalbin failed"
    fi
fi
done

echo "$goodcounter tests passed out of $(find . -name "*.s" | wc -l)"

